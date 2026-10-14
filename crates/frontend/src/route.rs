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
