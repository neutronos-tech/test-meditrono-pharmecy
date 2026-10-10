use crate::{
    route::Route,
    screen::{catalog, dashboard, history, inventory, login, purchase, sale, settings, stock},
    theme::{self, ThemeMode},
    widget::{feedback, sidebar},
};
use backend::{
    Backend, FullMedicineDownload, MedicineChangesDownload, RemoteError, SavedRemoteAuth,
    ServerClient,
};
use contracts::auth::{PharmacyAccount, PharmacyLoginResponse};
use iced::{
    Element, Length, Task,
    widget::{button, column, container, responsive, row, text},
};
use std::{fs, path::PathBuf};

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Pharmacy";
const APPLICATION: &str = "Pharmacy";

fn database_path() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or("Could not locate an application data directory.")?;
    fs::create_dir_all(dirs.data_dir())
        .map_err(|_| "Could not create the application data directory.".to_string())?;
    Ok(dirs.data_dir().join("pharmacy.db"))
}

pub struct App {
    backend: Option<Backend>,
    server: ServerClient,
    saved_auth: Option<SavedRemoteAuth>,
    authenticated_pharmacy: Option<PharmacyAccount>,
    catalog_ready: bool,
    catalog_loading: bool,
    catalog_error: Option<String>,
    catalog_total: Option<u64>,
    connection_error: Option<String>,
    route: Route,
    pub theme_mode: ThemeMode,
    fatal_error: Option<String>,
    login: login::State,
    dashboard: dashboard::State,
    purchase: purchase::State,
    stock: stock::State,
    inventory: inventory::State,
    sale: sale::State,
    history: history::State,
    catalog: catalog::State,
    settings: settings::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Route),
    SetTheme(ThemeMode),
    Logout,
    RetryConnection,
    RetryMedicineDownload,
    RememberedLoginChecked(SavedRemoteAuth, Result<PharmacyAccount, RemoteError>),
    Login(login::Message),
    LoginFinished(LoginResult),
    LogoutFinished,
    FullMedicinesDownloaded(Result<FullMedicineDownload, RemoteError>),
    MedicineChangesDownloaded(Result<MedicineChangesDownload, RemoteError>),
    Dashboard(dashboard::Message),
    Purchase(purchase::Message),
    Stock(stock::Message),
    Inventory(inventory::Message),
    Sale(sale::Message),
    History(history::Message),
    Catalog(catalog::Message),
    Settings(settings::Message),
}

#[derive(Clone)]
pub struct LoginResult(pub Result<PharmacyLoginResponse, RemoteError>);

impl std::fmt::Debug for LoginResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("LoginResult([redacted])")
    }
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let (backend, saved_auth, fatal_error) = match database_path().and_then(|path| {
            Backend::open(path)
                .map_err(|_| "Could not open the local pharmacy database.".to_string())
        }) {
            Ok(db) => match db.saved_remote_auth() {
                Ok(auth) => (Some(db), auth, None),
                Err(_) => (
                    None,
                    None,
                    Some("Could not read saved pharmacy account data.".into()),
                ),
            },
            Err(error) => (None, None, Some(error)),
        };
        let mut app = Self {
            backend,
            server: ServerClient::new(),
            saved_auth: None,
            authenticated_pharmacy: None,
            catalog_ready: false,
            catalog_loading: false,
            catalog_error: None,
            catalog_total: None,
            connection_error: None,
            route: Route::Dashboard,
            theme_mode: ThemeMode::Light,
            fatal_error,
            login: login::State::default(),
            dashboard: dashboard::State::default(),
            purchase: purchase::State::default(),
            stock: stock::State::default(),
            inventory: inventory::State::default(),
            sale: sale::State::default(),
            history: history::State::default(),
            catalog: catalog::State::default(),
            settings: settings::State,
        };
        let Some(saved) = saved_auth else {
            return (app, Task::none());
        };
        app.login.username = saved.pharmacy.username.clone();
        let client = app.server.clone();
        let saved_for_task = saved.clone();
        (
            app,
            Task::perform(
                async move {
                    let result = client.current_pharmacy(&saved_for_task.session_token).await;
                    (saved_for_task, result)
                },
                |(saved, result)| Message::RememberedLoginChecked(saved, result),
            ),
        )
    }

    fn full_catalog_task(&self, token: String) -> Task<Message> {
        let server = self.server.clone();
        Task::perform(
            async move { server.full_medicine_catalog(&token).await },
            Message::FullMedicinesDownloaded,
        )
    }

    fn changes_task(&self, token: String, after: u64) -> Task<Message> {
        let server = self.server.clone();
        Task::perform(
            async move { server.medicine_changes(&token, after).await },
            Message::MedicineChangesDownloaded,
        )
    }

    fn start_catalog_sync(&mut self) -> Task<Message> {
        let (Some(backend), Some(auth)) = (&self.backend, &self.saved_auth) else {
            return Task::none();
        };
        self.catalog_error = None;
        self.catalog_total = None;
        self.connection_error = None;
        let token = auth.session_token.clone();
        match backend.has_global_medicines() {
            Ok(false) => {
                self.catalog_ready = false;
                self.catalog_loading = true;
                self.full_catalog_task(token)
            }
            Ok(true) => match backend.medicine_revision() {
                Ok(revision) => {
                    self.catalog_ready = false;
                    self.catalog_loading = true;
                    self.changes_task(token, revision)
                }
                Err(_) => {
                    self.fatal_error = Some("Could not read the local medicine revision.".into());
                    Task::none()
                }
            },
            Err(_) => {
                self.fatal_error = Some("Could not inspect the local medicine database.".into());
                Task::none()
            }
        }
    }

    fn clear_auth(&mut self) {
        if let Some(backend) = &mut self.backend {
            let _ = backend.clear_remote_auth();
        }
        self.saved_auth = None;
        self.authenticated_pharmacy = None;
        self.catalog_loading = false;
        self.catalog_ready = false;
    }

    fn navigate(&mut self, route: Route) {
        if self.authenticated_pharmacy.is_none() || !self.catalog_ready {
            return;
        }
        let Some(backend) = &self.backend else {
            return;
        };
        self.route = route;
        match route {
            Route::Dashboard => dashboard::refresh(&mut self.dashboard, backend),
            Route::Purchase => purchase::refresh(&mut self.purchase, backend),
            Route::Stock => stock::refresh(&mut self.stock, backend),
            Route::Inventory => inventory::refresh(&mut self.inventory, backend),
            Route::Sale => sale::refresh(&mut self.sale, backend),
            Route::History => history::refresh(&mut self.history, backend),
            Route::Catalog => catalog::refresh(&mut self.catalog, backend),
            Route::Settings => settings::refresh(&mut self.settings),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RememberedLoginChecked(saved, result) => match result {
                Ok(pharmacy) => {
                    if let Some(backend) = &mut self.backend {
                        let _ = backend.set_saved_pharmacy(&pharmacy);
                    }
                    self.login.username = pharmacy.username.clone();
                    self.authenticated_pharmacy = Some(pharmacy);
                    self.saved_auth = Some(saved);
                    return self.start_catalog_sync();
                }
                Err(RemoteError::Unauthorized) => {
                    self.clear_auth();
                    self.login.error = Some("Your saved sign-in expired. Sign in again.".into());
                }
                Err(_) => {
                    self.saved_auth = Some(saved);
                    self.connection_error = Some(
                        "Could not verify your sign-in. Check your connection and retry.".into(),
                    );
                }
            },
            Message::Login(login::Message::Username(value)) => self.login.username = value,
            Message::Login(login::Message::Password(value)) => self.login.password = value,
            Message::Login(login::Message::Submit) => {
                self.login.error = None;
                if self.login.username.trim().is_empty() || self.login.password.is_empty() {
                    self.login.error = Some("Enter your username and password.".into());
                } else {
                    self.login.loading = true;
                    let username = self.login.username.trim().to_string();
                    let password = std::mem::take(&mut self.login.password);
                    let server = self.server.clone();
                    return Task::perform(
                        async move { server.login(username, password).await },
                        |result| Message::LoginFinished(LoginResult(result)),
                    );
                }
            }
            Message::LoginFinished(LoginResult(result)) => {
                self.login.loading = false;
                match result {
                    Ok(response) => {
                        if let Some(backend) = &mut self.backend
                            && backend.save_remote_auth(&response).is_err()
                        {
                            self.login.error =
                                Some("Could not save sign-in on this device.".into());
                            return Task::none();
                        }
                        self.login.username = response.pharmacy.username.clone();
                        self.saved_auth = Some(SavedRemoteAuth {
                            pharmacy: response.pharmacy.clone(),
                            session_token: response.session_token,
                        });
                        self.authenticated_pharmacy = Some(response.pharmacy);
                        self.login.error = None;
                        return self.start_catalog_sync();
                    }
                    Err(RemoteError::Unauthorized) => {
                        self.login.error = Some("Invalid username or password.".into());
                        self.login.password.clear();
                    }
                    Err(_) => {
                        self.login.error = Some("Could not connect to the Pharmacy server.".into());
                        self.login.password.clear();
                    }
                }
            }
            Message::FullMedicinesDownloaded(result) => {
                self.catalog_loading = false;
                match result {
                    Ok(download) => {
                        self.catalog_total = Some(download.total);
                        let revision = download.revision;
                        if let Some(backend) = &mut self.backend {
                            match backend.import_medicine_catalog(download.medicines, revision) {
                                Ok(()) => {
                                    self.catalog_loading = true;
                                    if let Some(auth) = &self.saved_auth {
                                        return self
                                            .changes_task(auth.session_token.clone(), revision);
                                    }
                                }
                                Err(_) => {
                                    self.catalog_error = Some(
                                        "Could not save the medicine database. Please retry."
                                            .into(),
                                    )
                                }
                            }
                        }
                    }
                    Err(RemoteError::Unauthorized) => self.clear_auth(),
                    Err(_) => {
                        self.catalog_error = Some("Could not load the medicine database.".into())
                    }
                }
            }
            Message::MedicineChangesDownloaded(result) => {
                self.catalog_loading = false;
                match result {
                    Ok(download) => {
                        if let Some(backend) = &mut self.backend {
                            match backend
                                .apply_medicine_changes(download.changes, download.revision)
                            {
                                Ok(()) => {
                                    self.catalog_ready = true;
                                    self.catalog_error = None;
                                }
                                Err(_) => {
                                    self.catalog_ready =
                                        backend.has_global_medicines().unwrap_or(false);
                                    self.connection_error = Some("Could not apply medicine updates. Local data remains available.".into());
                                }
                            }
                        }
                    }
                    Err(RemoteError::Unauthorized) => self.clear_auth(),
                    Err(_) => {
                        self.catalog_ready = self
                            .backend
                            .as_ref()
                            .is_some_and(|backend| backend.has_global_medicines().unwrap_or(false));
                        self.connection_error =
                            Some("Could not check medicine updates. Using local data.".into());
                    }
                }
            }
            Message::RetryConnection => {
                if let Some(saved) = &self.saved_auth {
                    let client = self.server.clone();
                    let saved = saved.clone();
                    return Task::perform(
                        async move {
                            let result = client.current_pharmacy(&saved.session_token).await;
                            (saved, result)
                        },
                        |(saved, result)| Message::RememberedLoginChecked(saved, result),
                    );
                }
            }
            Message::RetryMedicineDownload => return self.start_catalog_sync(),
            Message::Navigate(route) => self.navigate(route),
            Message::SetTheme(mode) => self.theme_mode = mode,
            Message::Logout => {
                if let Some(saved) = self.saved_auth.clone() {
                    let server = self.server.clone();
                    let token = saved.session_token;
                    self.clear_auth();
                    return Task::perform(
                        async move {
                            let _ = server.logout(&token).await;
                        },
                        |_| Message::LogoutFinished,
                    );
                }
                self.clear_auth();
            }
            Message::LogoutFinished => {}
            Message::Dashboard(message) => match message {
                dashboard::Message::Refresh => {
                    if let Some(backend) = &self.backend {
                        dashboard::refresh(&mut self.dashboard, backend);
                    }
                }
                dashboard::Message::QuickAction(route) => self.navigate(route),
            },
            Message::Purchase(message) => {
                if let (Some(backend), Some(pharmacy)) =
                    (&mut self.backend, &self.authenticated_pharmacy)
                {
                    purchase::update(&mut self.purchase, message, backend, pharmacy);
                }
            }
            Message::Stock(message) => {
                if let (Some(backend), Some(pharmacy)) =
                    (&mut self.backend, &self.authenticated_pharmacy)
                {
                    stock::update(&mut self.stock, message, backend, pharmacy);
                }
            }
            Message::Inventory(message) => {
                if let Some(backend) = &mut self.backend {
                    inventory::update(&mut self.inventory, message, backend);
                }
            }
            Message::Sale(message) => {
                if let (Some(backend), Some(pharmacy)) =
                    (&mut self.backend, &self.authenticated_pharmacy)
                {
                    sale::update(&mut self.sale, message, backend, pharmacy);
                }
            }
            Message::History(message) => {
                if let (Some(backend), Some(pharmacy)) =
                    (&mut self.backend, &self.authenticated_pharmacy)
                {
                    history::update(&mut self.history, message, backend, pharmacy);
                }
            }
            Message::Catalog(message) => {
                if let (Some(backend), Some(pharmacy)) =
                    (&mut self.backend, &self.authenticated_pharmacy)
                {
                    catalog::update(&mut self.catalog, message, backend, pharmacy);
                }
            }
            Message::Settings(message) => match message {},
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        if let Some(error) = &self.fatal_error {
            return container(text(error))
                .padding(theme::spacing::XXL)
                .center(Length::Fill)
                .into();
        }
        if self.backend.is_none() {
            return container(text("Database unavailable"))
                .center(Length::Fill)
                .into();
        }
        if self.authenticated_pharmacy.is_none() {
            if let Some(error) = &self.connection_error {
                return container(column![
                    text("Pharmacy"),
                    feedback::error_banner(error),
                    button("Retry").on_press(Message::RetryConnection)
                ])
                .padding(theme::spacing::XXL)
                .center(Length::Fill)
                .into();
            }
            return login::view(&self.login).map(Message::Login);
        }
        if !self.catalog_ready {
            let mut content = column![
                text("Pharmacy").size(30),
                text("Loading medicine database…").size(20),
                text("Downloading the catalogue for offline use.").style(theme::text::muted)
            ]
            .spacing(theme::spacing::LG)
            .align_x(iced::Alignment::Center);
            if self.catalog_loading {
                content = content.push(iced_aw::Spinner::new().width(32).height(32));
            }
            if let Some(total) = self.catalog_total {
                content = content.push(text(format!(
                    "{total} medicines downloaded. Checking updates…"
                )));
            }
            if let Some(error) = &self.catalog_error {
                content = content.push(feedback::error_banner(error)).push(
                    button("Retry")
                        .style(theme::button::primary)
                        .on_press(Message::RetryMedicineDownload),
                );
            }
            return container(content).center(Length::Fill).into();
        }
        let Some(pharmacy) = self.authenticated_pharmacy.as_ref() else {
            return container(text("Sign in required"))
                .center(Length::Fill)
                .into();
        };
        responsive(move |size| {
            let page: Element<'_, Message> = match self.route {
                Route::Dashboard => {
                    dashboard::view(&self.dashboard, pharmacy).map(Message::Dashboard)
                }
                Route::Purchase => purchase::view(&self.purchase).map(Message::Purchase),
                Route::Stock => stock::view(&self.stock).map(Message::Stock),
                Route::Inventory => inventory::view(&self.inventory).map(Message::Inventory),
                Route::Sale => sale::view(&self.sale).map(Message::Sale),
                Route::History => history::view(&self.history, pharmacy).map(Message::History),
                Route::Catalog => catalog::view(&self.catalog).map(Message::Catalog),
                Route::Settings => settings::view(pharmacy).map(Message::Settings),
            };
            let page = if let Some(error) = &self.connection_error {
                column![feedback::info_banner(error), page].into()
            } else {
                page
            };
            row![
                sidebar::view(pharmacy, self.route, size.width < 1050.0, self.theme_mode),
                container(page)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::container::app)
            ]
            .height(Length::Fill)
            .into()
        })
        .into()
    }
}
