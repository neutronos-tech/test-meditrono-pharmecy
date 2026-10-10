use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicineCatalogItem {
    pub id: Uuid,
    pub name: String,
    pub generic_name: Option<String>,
    pub strength: Option<String>,
    pub dosage_form: Option<String>,
    pub manufacturer: Option<String>,
    pub package_description: Option<String>,
    pub reference_price: Option<Decimal>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicineChange {
    pub revision: u64,
    pub medicine: MedicineCatalogItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: Uuid,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medicine {
    pub id: Uuid,
    pub name: String,
    pub generic_name: Option<String>,
    pub strength: Option<String>,
    pub dosage_form: Option<String>,
    pub manufacturer: Option<String>,
    pub package_description: Option<String>,
    pub reference_price: Option<Decimal>,
    pub selling_price: Option<Decimal>,
    pub active: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSupplierInput {
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMedicineInput {
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSellingPriceInput {
    pub medicine_id: Uuid,
    pub selling_price: Option<Decimal>,
}
