#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Dashboard,
    Purchase,
    Stock,
    Inventory,
    Sale,
    History,
    Catalog,
    Settings,
}
impl Route {
    pub const ALL: [Route; 8] = [
        Route::Dashboard,
        Route::Purchase,
        Route::Stock,
        Route::Inventory,
        Route::Sale,
        Route::History,
        Route::Catalog,
        Route::Settings,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Purchase => "Purchase",
            Self::Stock => "Stock Intake",
            Self::Inventory => "Inventory",
            Self::Sale => "Sales",
            Self::History => "History",
            Self::Catalog => "Catalog",
            Self::Settings => "Settings",
        }
    }
}
