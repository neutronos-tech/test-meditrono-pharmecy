use super::*;
use contracts::{
    auth::PharmacyAccount,
    catalog::{CreateMedicineInput, CreateSupplierInput},
    history::HistoryFilter,
    purchase::{PurchaseInput, PurchaseItemInput},
    sale::{SaleInput, SaleItemInput},
    stock::{StockReceiptInput, StockReceiptLineInput},
};
use rust_decimal::Decimal;
use tempfile::TempDir;
use time::{Date, Duration, OffsetDateTime};
use uuid::Uuid;

fn setup() -> (TempDir, Backend, PharmacyAccount) {
    let dir = tempfile::tempdir().unwrap();
    let backend = Backend::open(dir.path().join("pharmacy.db")).unwrap();
    let pharmacy = PharmacyAccount {
        id: Uuid::now_v7(),
        username: "sample pharmacy".into(),
    };
    (dir, backend, pharmacy)
}
fn count(db: &Backend, table: &str) -> i64 {
    db.db
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn catalog(db: &mut Backend, pharmacy: &PharmacyAccount) -> (Uuid, Uuid) {
    let supplier = db
        .create_supplier(
            pharmacy,
            CreateSupplierInput {
                name: "Supplier".into(),
            },
        )
        .unwrap();
    let medicine = db
        .create_medicine(
            pharmacy,
            CreateMedicineInput {
                name: "Medicine".into(),
            },
        )
        .unwrap();
    (supplier.id, medicine.id)
}
fn purchase(
    db: &mut Backend,
    pharmacy: &PharmacyAccount,
    supplier: Uuid,
    medicine: Uuid,
    invoice: &str,
    quantity: u32,
) -> contracts::purchase::Purchase {
    db.create_purchase(
        pharmacy,
        PurchaseInput {
            supplier_id: supplier,
            invoice_number: invoice.into(),
            items: vec![PurchaseItemInput {
                medicine_id: medicine,
                quantity,
                unit_cost: Decimal::new(1250, 2),
            }],
        },
    )
    .unwrap()
}
fn receive(
    db: &mut Backend,
    pharmacy: &PharmacyAccount,
    purchase: &contracts::purchase::Purchase,
    batch: &str,
    quantity: u32,
    expiry: Date,
) -> Uuid {
    db.receive_stock(
        pharmacy,
        StockReceiptInput {
            purchase_id: purchase.id,
            lines: vec![StockReceiptLineInput {
                purchase_item_id: purchase.items[0].id,
                quantity,
                batch_number: batch.into(),
                expiry_date: expiry,
            }],
        },
    )
    .unwrap()
}
fn future(days: i64) -> Date {
    (OffsetDateTime::now_utc() + Duration::days(days))
        .to_offset(time::UtcOffset::from_hms(6, 0, 0).unwrap())
        .date()
}

#[test]
fn creates_database_and_migration() {
    let (dir, db, _) = setup();
    assert!(dir.path().join("pharmacy.db").exists());
    assert_eq!(
        db.db
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        db.db
            .query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(db.medicine_revision().unwrap(), 0);
}
#[test]
fn catalog_purchase_and_duplicate_outbox() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    assert_eq!(db.search_suppliers("Supp", 10).unwrap().len(), 1);
    assert_eq!(db.search_medicines("Med", 10).unwrap().len(), 1);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "INV-1", 10);
    assert_eq!(db.purchase(p.id).unwrap().items.len(), 1);
    assert_eq!(count(&db, "purchases"), 1);
    assert_eq!(count(&db, "purchase_items"), 1);
    let pending = db.pending_sync(100).unwrap();
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[2].entity_type, "purchase");
    assert_eq!(pending[2].entity_id, p.id);
    assert!(matches!(
        db.create_purchase(
            &pharmacy,
            PurchaseInput {
                supplier_id: supplier,
                invoice_number: "INV-1".into(),
                items: vec![PurchaseItemInput {
                    medicine_id: medicine,
                    quantity: 1,
                    unit_cost: Decimal::ONE
                }]
            }
        ),
        Err(Error::DuplicateInvoice)
    ));
    assert_eq!(count(&db, "purchases"), 1);
    assert_eq!(db.pending_sync(100).unwrap().len(), 3);
}
#[test]
fn purchase_rolls_back_on_missing_medicine() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let result = db.create_purchase(
        &pharmacy,
        PurchaseInput {
            supplier_id: supplier,
            invoice_number: "I".into(),
            items: vec![
                PurchaseItemInput {
                    medicine_id: medicine,
                    quantity: 1,
                    unit_cost: Decimal::ONE,
                },
                PurchaseItemInput {
                    medicine_id: Uuid::now_v7(),
                    quantity: 1,
                    unit_cost: Decimal::ONE,
                },
            ],
        },
    );
    assert!(matches!(result, Err(Error::NotFound)));
    assert_eq!(count(&db, "operations"), 0);
    assert_eq!(count(&db, "purchases"), 0);
    assert_eq!(count(&db, "sync_outbox"), 2);
}
#[test]
fn receive_reversal_and_negative_guard() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 10);
    let receipt = receive(&mut db, &pharmacy, &p, "B1", 6, future(100));
    assert_eq!(db.receivable_purchase(p.id).unwrap().items[0].remaining, 4);
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 6);
    assert_eq!(db.pending_sync(100).unwrap().len(), 4);
    let result = db.receive_stock(
        &pharmacy,
        StockReceiptInput {
            purchase_id: p.id,
            lines: vec![StockReceiptLineInput {
                purchase_item_id: p.items[0].id,
                quantity: 5,
                batch_number: "B1".into(),
                expiry_date: future(100),
            }],
        },
    );
    assert!(matches!(result, Err(Error::ReceiptExceedsPurchase)));
    assert_eq!(db.pending_sync(100).unwrap().len(), 4);
    let reversal = db.reverse_stock_receipt(&pharmacy, receipt).unwrap();
    assert_ne!(reversal, receipt);
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 0);
    assert_eq!(db.receivable_purchase(p.id).unwrap().items[0].remaining, 10);
    assert!(matches!(
        db.reverse_stock_receipt(&pharmacy, receipt),
        Err(Error::AlreadyReversed)
    ));
}
#[test]
fn stock_reversal_rejects_consumed_stock() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 10);
    let receipt = receive(&mut db, &pharmacy, &p, "B1", 10, future(100));
    db.create_sale(
        &pharmacy,
        SaleInput {
            items: vec![SaleItemInput {
                medicine_id: medicine,
                quantity: 1,
                unit_price: Decimal::ONE,
            }],
        },
    )
    .unwrap();
    assert!(matches!(
        db.reverse_stock_receipt(&pharmacy, receipt),
        Err(Error::StockWouldGoNegative)
    ));
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 9);
    assert_eq!(count(&db, "operations"), 3);
}
#[test]
fn fefo_sale_rollback_reversal_and_outbox() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 20);
    receive(&mut db, &pharmacy, &p, "LATE", 10, future(120));
    receive(&mut db, &pharmacy, &p, "EARLY", 10, future(30));
    let before = db.inventory_detail(medicine).unwrap();
    assert_eq!(before.total_quantity, 20);
    let fail = db.create_sale(
        &pharmacy,
        SaleInput {
            items: vec![SaleItemInput {
                medicine_id: medicine,
                quantity: 21,
                unit_price: Decimal::new(200, 2),
            }],
        },
    );
    assert!(matches!(fail, Err(Error::InsufficientStock)));
    assert_eq!(count(&db, "sales"), 0);
    assert_eq!(count(&db, "sale_items"), 0);
    assert_eq!(count(&db, "sync_outbox"), 5);
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 20);
    let sale = db
        .create_sale(
            &pharmacy,
            SaleInput {
                items: vec![SaleItemInput {
                    medicine_id: medicine,
                    quantity: 12,
                    unit_price: Decimal::new(200, 2),
                }],
            },
        )
        .unwrap();
    assert_eq!(count(&db, "sales"), 1);
    assert_eq!(count(&db, "sync_outbox"), 6);
    let after = db.inventory_detail(medicine).unwrap();
    assert_eq!(
        after
            .batches
            .iter()
            .find(|b| b.batch_number == "EARLY")
            .unwrap()
            .quantity_on_hand,
        0
    );
    assert_eq!(
        after
            .batches
            .iter()
            .find(|b| b.batch_number == "LATE")
            .unwrap()
            .quantity_on_hand,
        8
    );
    assert!(matches!(
        db.reverse_sale(&pharmacy, sale.id),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        db.reverse_sale(&pharmacy, p.operation_id),
        Err(Error::NotFound)
    ));
    db.reverse_sale(&pharmacy, sale.operation_id).unwrap();
    let restored = db.inventory_detail(medicine).unwrap();
    assert_eq!(restored.total_quantity, 20);
    assert_eq!(
        restored
            .batches
            .iter()
            .find(|b| b.batch_number == "EARLY")
            .unwrap()
            .quantity_on_hand,
        10
    );
    assert!(matches!(
        db.reverse_sale(&pharmacy, sale.operation_id),
        Err(Error::AlreadyReversed)
    ));
}
#[test]
fn multiline_sale_rolls_back_and_history_dashboard_work() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 5);
    receive(&mut db, &pharmacy, &p, "B", 5, future(100));
    let fail = db.create_sale(
        &pharmacy,
        SaleInput {
            items: vec![
                SaleItemInput {
                    medicine_id: medicine,
                    quantity: 2,
                    unit_price: Decimal::ONE,
                },
                SaleItemInput {
                    medicine_id: medicine,
                    quantity: 4,
                    unit_price: Decimal::ONE,
                },
            ],
        },
    );
    assert!(matches!(fail, Err(Error::InsufficientStock)));
    assert_eq!(count(&db, "sales"), 0);
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 5);
    let sale = db
        .create_sale(
            &pharmacy,
            SaleInput {
                items: vec![SaleItemInput {
                    medicine_id: medicine,
                    quantity: 2,
                    unit_price: Decimal::new(1250, 2),
                }],
            },
        )
        .unwrap();
    let dashboard = db.dashboard().unwrap();
    assert_eq!(dashboard.today_sales_count, 1);
    assert_eq!(dashboard.today_sales_total, Decimal::new(2500, 2));
    assert_eq!(dashboard.medicines_in_stock, 1);
    assert_eq!(dashboard.low_stock_medicines, 1);
    let history = db.history(HistoryFilter::Sales, 10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation_id, sale.operation_id);
    assert!(history[0].reversible);
    db.reverse_sale(&pharmacy, sale.operation_id).unwrap();
    assert_eq!(db.dashboard().unwrap().today_sales_count, 0);
    assert_eq!(db.history(HistoryFilter::Reversals, 10).unwrap().len(), 1);
}
#[test]
fn expired_stock_visible_but_not_sellable() {
    let (_, mut db, pharmacy) = setup();
    let (_, medicine) = catalog(&mut db, &pharmacy);
    let batch = Uuid::now_v7();
    db.db.execute("INSERT INTO inventory_batches(id,medicine_id,batch_number,expiry_date,quantity_on_hand) VALUES(?1,?2,'OLD',?3,5)",rusqlite::params![batch.to_string(),medicine.to_string(),database::date_to_db(future(-1))]).unwrap();
    assert_eq!(db.inventory("", 10).unwrap()[0].total_quantity, 5);
    assert!(db.sellable_inventory("", 10).unwrap().is_empty());
    assert!(matches!(
        db.create_sale(
            &pharmacy,
            SaleInput {
                items: vec![SaleItemInput {
                    medicine_id: medicine,
                    quantity: 1,
                    unit_price: Decimal::ONE
                }]
            }
        ),
        Err(Error::InsufficientStock)
    ));
}
#[test]
fn sync_ack_and_revision() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    purchase(&mut db, &pharmacy, supplier, medicine, "I", 1);
    let pending = db.pending_sync(10).unwrap();
    assert_eq!(pending.len(), 3);
    db.acknowledge_sync(&pending.iter().map(|p| p.change_id).collect::<Vec<_>>())
        .unwrap();
    assert!(db.pending_sync(10).unwrap().is_empty());
    db.set_medicine_revision(42).unwrap();
    assert_eq!(db.medicine_revision().unwrap(), 42);
    assert!(matches!(db.set_medicine_revision(41), Err(Error::Conflict)));
    assert_eq!(db.medicine_revision().unwrap(), 42);
}

#[test]
fn receipt_rejects_batch_conflict_and_rolls_back_all_lines() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 10);
    receive(&mut db, &pharmacy, &p, "B", 2, future(100));
    let before_outbox = count(&db, "sync_outbox");
    let result = db.receive_stock(
        &pharmacy,
        StockReceiptInput {
            purchase_id: p.id,
            lines: vec![
                StockReceiptLineInput {
                    purchase_item_id: p.items[0].id,
                    quantity: 2,
                    batch_number: "C".into(),
                    expiry_date: future(100),
                },
                StockReceiptLineInput {
                    purchase_item_id: p.items[0].id,
                    quantity: 2,
                    batch_number: "B".into(),
                    expiry_date: future(101),
                },
            ],
        },
    );
    assert!(matches!(result, Err(Error::BatchMetadataConflict)));
    assert_eq!(db.inventory_detail(medicine).unwrap().total_quantity, 2);
    assert_eq!(db.receivable_purchase(p.id).unwrap().items[0].remaining, 8);
    assert_eq!(count(&db, "sync_outbox"), before_outbox);
    assert_eq!(count(&db, "stock_inputs"), 1);
}

#[test]
fn catalog_price_and_reopen_persist() {
    let (dir, mut db, pharmacy) = setup();
    let (_, medicine) = catalog(&mut db, &pharmacy);
    db.set_selling_price(
        &pharmacy,
        contracts::catalog::SetSellingPriceInput {
            medicine_id: medicine,
            selling_price: Some(Decimal::new(1050, 2)),
        },
    )
    .unwrap();
    assert_eq!(
        db.inventory_detail(medicine)
            .unwrap()
            .medicine
            .selling_price,
        Some(Decimal::new(1050, 2))
    );
    assert_eq!(
        db.pending_sync(100)
            .unwrap()
            .iter()
            .filter(|e| e.entity_type == "medicine")
            .count(),
        2
    );
    drop(db);
    let db = Backend::open(dir.path().join("pharmacy.db")).unwrap();
    assert_eq!(db.medicine_revision().unwrap(), 0);
    assert_eq!(
        db.inventory_detail(medicine)
            .unwrap()
            .medicine
            .selling_price,
        Some(Decimal::new(1050, 2))
    );
}

#[test]
fn receipt_rejects_expired_business_date() {
    let (_, mut db, pharmacy) = setup();
    let (supplier, medicine) = catalog(&mut db, &pharmacy);
    let p = purchase(&mut db, &pharmacy, supplier, medicine, "I", 3);
    let before = count(&db, "sync_outbox");
    let result = db.receive_stock(
        &pharmacy,
        StockReceiptInput {
            purchase_id: p.id,
            lines: vec![StockReceiptLineInput {
                purchase_item_id: p.items[0].id,
                quantity: 1,
                batch_number: "OLD".into(),
                expiry_date: future(-2),
            }],
        },
    );
    assert!(matches!(result, Err(Error::Conflict)));
    assert_eq!(count(&db, "stock_inputs"), 0);
    assert_eq!(count(&db, "sync_outbox"), before);
}

#[test]
fn low_stock_takes_priority_over_near_expiry() {
    let (_, mut db, pharmacy) = setup();
    let (_, medicine) = catalog(&mut db, &pharmacy);
    db.db
        .execute(
            "INSERT INTO inventory_batches(id,medicine_id,batch_number,expiry_date,quantity_on_hand) VALUES(?1,?2,'SOON',?3,2)",
            rusqlite::params![Uuid::now_v7().to_string(), medicine.to_string(), database::date_to_db(future(1))],
        )
        .unwrap();
    let item = &db.inventory("", 10).unwrap()[0];
    assert_eq!(item.total_quantity, 2);
    assert_eq!(item.status, contracts::inventory::InventoryStatus::LowStock);
}

fn global_medicine(id: Uuid, name: &str) -> contracts::catalog::MedicineCatalogItem {
    contracts::catalog::MedicineCatalogItem {
        id,
        name: name.into(),
        generic_name: Some("Paracetamol".into()),
        strength: Some("500 mg".into()),
        dosage_form: Some("Tablet".into()),
        manufacturer: None,
        package_description: None,
        reference_price: Some(Decimal::new(1000, 2)),
        active: true,
    }
}

#[test]
fn full_catalog_import_and_changes_preserve_local_selling_price_without_outbox_echo() {
    let (_, mut db, pharmacy) = setup();
    let id = Uuid::now_v7();
    assert!(!db.has_global_medicines().unwrap());
    db.import_medicine_catalog(vec![global_medicine(id, "Central medicine")], 0)
        .unwrap();
    assert!(db.has_global_medicines().unwrap());
    assert_eq!(db.medicine_revision().unwrap(), 0);
    assert_eq!(count(&db, "sync_outbox"), 0);
    db.set_selling_price(
        &pharmacy,
        contracts::catalog::SetSellingPriceInput {
            medicine_id: id,
            selling_price: Some(Decimal::new(1250, 2)),
        },
    )
    .unwrap();
    let mut updated = global_medicine(id, "Renamed centrally");
    updated.reference_price = Some(Decimal::new(1500, 2));
    updated.active = false;
    db.apply_medicine_changes(
        vec![contracts::catalog::MedicineChange {
            revision: 1,
            medicine: updated.clone(),
        }],
        1,
    )
    .unwrap();
    assert_eq!(db.medicine_revision().unwrap(), 1);
    let detail = db.inventory_detail(id).unwrap();
    assert_eq!(detail.medicine.name, updated.name);
    assert_eq!(detail.medicine.reference_price, updated.reference_price);
    assert_eq!(detail.medicine.selling_price, Some(Decimal::new(1250, 2)));
    assert!(!detail.medicine.active);
    assert!(db.search_medicines("Renamed", 10).unwrap().is_empty());
    assert_eq!(count(&db, "sync_outbox"), 1);
    db.apply_medicine_changes(
        vec![contracts::catalog::MedicineChange {
            revision: 1,
            medicine: updated,
        }],
        1,
    )
    .unwrap();
    assert_eq!(count(&db, "sync_outbox"), 1);
}

#[test]
fn failed_catalog_import_rolls_back_rows_and_revision() {
    let (_, mut db, _) = setup();
    db.db.execute_batch("CREATE TRIGGER fail_global BEFORE INSERT ON medicines WHEN NEW.name='FAIL' BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
    let result = db.import_medicine_catalog(
        vec![
            global_medicine(Uuid::now_v7(), "GOOD"),
            global_medicine(Uuid::now_v7(), "FAIL"),
        ],
        7,
    );
    assert!(result.is_err());
    assert_eq!(count(&db, "medicines"), 0);
    assert_eq!(db.medicine_revision().unwrap(), 0);
    assert!(!db.has_global_medicines().unwrap());
    assert_eq!(count(&db, "sync_outbox"), 0);
}

#[test]
fn failed_incremental_apply_rolls_back_rows_and_revision() {
    let (_, mut db, _) = setup();
    let id = Uuid::now_v7();
    db.import_medicine_catalog(vec![global_medicine(id, "Original")], 2)
        .unwrap();
    db.db.execute_batch("CREATE TRIGGER fail_change BEFORE INSERT ON medicines WHEN NEW.name='FAIL' BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();

    let result = db.apply_medicine_changes(
        vec![
            contracts::catalog::MedicineChange {
                revision: 3,
                medicine: global_medicine(id, "Updated"),
            },
            contracts::catalog::MedicineChange {
                revision: 4,
                medicine: global_medicine(Uuid::now_v7(), "FAIL"),
            },
        ],
        4,
    );

    assert!(result.is_err());
    assert_eq!(db.medicine_revision().unwrap(), 2);
    let name: String = db
        .db
        .query_row(
            "SELECT name FROM medicines WHERE id=?1",
            [id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(name, "Original");
    assert_eq!(count(&db, "medicines"), 1);
    assert_eq!(count(&db, "sync_outbox"), 0);
}

#[test]
fn existing_version_one_database_migrates_to_medicine_revision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("old.db");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch(include_str!("../../../migrations/0001_initial.sql"))
        .unwrap();
    db.execute_batch("PRAGMA user_version=1").unwrap();
    drop(db);
    let backend = Backend::open(&path).unwrap();
    assert_eq!(
        backend
            .db
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(backend.medicine_revision().unwrap(), 0);
}

#[test]
fn search_matches_brand_and_generic_names_ignoring_ascii_case() {
    let (_, mut db, _) = setup();
    db.import_medicine_catalog(vec![global_medicine(Uuid::now_v7(), "Napa")], 1)
        .unwrap();
    assert_eq!(db.search_medicines("nap", 10).unwrap().len(), 1);
    assert_eq!(db.search_medicines("PARAC", 10).unwrap().len(), 1);
    assert!(db.search_medicines("zzz", 10).unwrap().is_empty());
}

#[test]
fn search_orders_by_name_and_honors_limit() {
    let (_, mut db, _) = setup();
    let medicines = ["Cetirizine", "Ace", "Bisoprolol"]
        .into_iter()
        .map(|name| global_medicine(Uuid::now_v7(), name))
        .collect();
    db.import_medicine_catalog(medicines, 1).unwrap();
    let names: Vec<String> = db
        .search_medicines("", 2)
        .unwrap()
        .into_iter()
        .map(|medicine| medicine.name)
        .collect();
    assert_eq!(names, ["Ace", "Bisoprolol"]);
}

#[test]
fn search_hides_medicines_deactivated_by_changes() {
    let (_, mut db, _) = setup();
    let id = Uuid::now_v7();
    db.import_medicine_catalog(vec![global_medicine(id, "Napa")], 1)
        .unwrap();
    let mut retired = global_medicine(id, "Napa");
    retired.active = false;
    db.apply_medicine_changes(
        vec![contracts::catalog::MedicineChange {
            revision: 2,
            medicine: retired,
        }],
        2,
    )
    .unwrap();
    assert!(db.search_medicines("Napa", 10).unwrap().is_empty());
}

#[test]
fn search_includes_locally_created_medicines() {
    let (_, mut db, pharmacy) = setup();
    db.create_medicine(
        &pharmacy,
        CreateMedicineInput {
            name: "Local Syrup".into(),
        },
    )
    .unwrap();
    assert_eq!(db.search_medicines("Syrup", 10).unwrap().len(), 1);
}

#[test]
fn search_finds_rows_in_a_large_import() {
    let (_, mut db, _) = setup();
    let medicines = (0..5000u32)
        .map(|index| global_medicine(Uuid::now_v7(), &format!("Brand {index:05}")))
        .collect();
    db.import_medicine_catalog(medicines, 1).unwrap();
    assert_eq!(db.search_medicines("Brand 04999", 10).unwrap().len(), 1);
    assert_eq!(db.search_medicines("Brand", 100).unwrap().len(), 100);
}

/// Known gap: user input is not escaped, so `%` and `_` act as LIKE wildcards.
/// This test pins the current behaviour until escaping is added.
#[test]
fn search_currently_treats_like_wildcards_in_input_as_wildcards() {
    let (_, mut db, _) = setup();
    db.import_medicine_catalog(
        vec![
            global_medicine(Uuid::now_v7(), "Napa"),
            global_medicine(Uuid::now_v7(), "Ace"),
        ],
        1,
    )
    .unwrap();
    assert_eq!(db.search_medicines("%", 10).unwrap().len(), 2);
}
