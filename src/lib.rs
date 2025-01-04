pub mod handlers {
    pub mod users {
        pub mod authorize;
        pub mod login;
        pub mod new;
    }
}

pub mod models {
    pub mod appstate;
    pub mod user;
}

pub mod util{
    pub mod validation;
    pub mod jwt {
        pub mod generate;
    }
}



