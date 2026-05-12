pub mod api {
    pub mod api;
    pub mod router;
}

pub mod auth;

pub mod backup;

pub mod base {
    pub mod args;
    pub mod config;
    pub mod log;
    pub mod signal;
}

pub mod bot {
    pub mod bot;
    pub mod common;
    pub mod router;
}

pub mod kv {
    pub mod backup;
    pub mod bot;
    pub mod kv;
    pub mod meta;
}

pub mod logs;

pub mod services;
