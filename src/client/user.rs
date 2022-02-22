use super::{ConnectionSet, ConnectionInfo, ActiveConnection, OpenedScripts, OpenedFile};
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use crate::ui::QueriesWindow;
use std::cell::RefCell;
use crate::React;
use std::rc::Rc;
use std::ops::Deref;
use std::thread;
use gtk4::*;
use gtk4::prelude::*;
use crate::client::QueriesClient;
use std::convert::TryInto;
// use sha2::Digest;
use std::hash::Hash;
use std::path::Path;
use base64;
use crate::ui::Certificate;

// use_litcrypt!("key");
// lc!("String name")

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub scheme : String,
    pub font_family : String,
    pub font_size : i32,
    pub show_line_numbers : bool,
    pub highlight_current_line : bool
}

impl Default for EditorSettings {

    fn default() -> Self {
        Self {
            scheme : String::from("Adwaita"),
            font_family : String::from("Ubuntu Mono"),
            font_size : 16,
            show_line_numbers : false,
            highlight_current_line : false
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    pub row_limit : i32,
    pub column_limit : i32,
    pub execution_interval : i32,
    pub statement_timeout : i32
}

impl Default for ExecutionSettings {

    fn default() -> Self {
        Self {
            row_limit : 500,
            column_limit : 25,
            execution_interval : 1,
            statement_timeout : 5
        }
    }

}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct UserState {

    pub main_handle_pos : i32,

    pub side_handle_pos : i32,

    pub window_width : i32,

    pub window_height : i32,

    pub scripts : Vec<OpenedFile>,

    #[serde(serialize_with = "ser_conns")]
    #[serde(deserialize_with = "deser_conns")]
    pub conns : Vec<ConnectionInfo>,

    pub templates : Vec<String>,

    pub selected_template : usize,

    #[serde(skip)]
    pub unmatched_certs : Vec<Certificate>,

    pub editor : EditorSettings,

    pub execution : ExecutionSettings

}

use serde::Deserializer;
use serde::Serializer;

const KEY : [u8; 32] = [
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8)
];

const NONCE : [u8; 24] = [
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8),
    const_random::const_random!(u8)
];

fn ser_conns<S>(conns : &Vec<ConnectionInfo>, ser : S) -> Result<S::Ok, S::Error>
    where S: Serializer
{
    use chacha20poly1305::aead::NewAead;
    use chacha20poly1305::aead::Aead;
    let plain = serde_json::to_string(conns).unwrap();
    let cipher = chacha20poly1305::XChaCha20Poly1305::new((&KEY).into());
    let enc : Vec<u8> = cipher
        .encrypt((&NONCE).into(), plain.as_ref())
        .unwrap();
    let enc_base64 : String = base64::encode(enc);
    enc_base64.serialize(ser)
}

fn deser_conns<'de, D>(deser : D) -> Result<Vec<ConnectionInfo>, D::Error>
    where D: Deserializer<'de>
{
    use chacha20poly1305::aead::NewAead;
    use chacha20poly1305::aead::Aead;
    let enc_base64 : String = <String as Deserialize>::deserialize(deser)?;
    let enc_bytes : Vec<u8> = base64::decode(enc_base64).unwrap();
    let cipher = chacha20poly1305::XChaCha20Poly1305::new((&KEY).into());
    match cipher.decrypt((&NONCE).into(), enc_bytes.as_ref()) {
        Ok(dec) => {
            let plain = String::from_utf8(dec).unwrap();
            let out : Vec<ConnectionInfo> = serde_json::from_str(&plain).unwrap();
            Ok(out)
        },
        Err(_) => {
            // The decoding should fail whenever queries is re-built, since a
            // new random key will be generated. Just clean the user connection
            // state in this case.
            Ok(Vec::new())
        }
    }
}

#[derive(Clone)]
pub struct SharedUserState(Rc<RefCell<UserState>>);

impl Deref for SharedUserState {

    type Target = RefCell<UserState>;

    fn deref(&self) -> &RefCell<UserState> {
        &self.0
    }

}

impl Default for SharedUserState {

    fn default() -> Self {
        SharedUserState(Rc::new(RefCell::new(UserState {
            main_handle_pos : 100,
            side_handle_pos : 400,
            window_width : 1024,
            window_height : 768,
            selected_template : 0,
            ..Default::default()
        })))
    }

}

impl SharedUserState {

    /// Attempts to open UserState by deserializing it from a JSON path.
    /// This is a blocking operation.
    pub fn open(path : &str) -> Option<SharedUserState> {
        let state : UserState = serde_json::from_reader(File::open(path).ok()?).ok()?;
        Some(SharedUserState(Rc::new(RefCell::new(state))))
    }

}

/// Saves the state to the given path by spawning a thread. This is
/// a nonblocking operation.
pub fn persist_user_preferences(user_state : &SharedUserState, path : &str) -> thread::JoinHandle<bool> {
    let mut state : UserState = user_state.borrow().clone();
    state.scripts.iter_mut().for_each(|s| { s.content.as_mut().map(|c| c.clear() ); } );
    let path = path.to_string();

    // TODO filter repeated scripts and connections

    thread::spawn(move|| {
        if let Ok(f) = File::create(&path) {
            serde_json::to_writer_pretty(f, &state).is_ok()
        } else {
            false
        }
    })
}

/*impl React<super::ActiveConnection> for SharedUserState {

    fn react(&self, conn : &ActiveConnection) {
        conn.connect_db_connected(move |opt_db_info| {
            // Connection already present? If not, add it and save.
            if let Some(info) = opt_db_info {

            }
        });
    }

}*/

/*impl React<super::ConnectionSet> for SharedUserState {

    fn react(&self, set : &ConnectionSet) {
        set.connect_removed({
            let state = self.clone();
            move |ix| {
                let mut state = state.borrow_mut();
                if ix >= 0 {
                    state.conns.remove(ix as usize);
                }
            }
        });
        set.connect_updated({
            let state = self.clone();
            move |(ix, info)| {
                let mut state = state.borrow_mut();
                state.conns[ix as usize] = info;
            }
        });
        set.connect_added({
            let state = self.clone();
            move |conn| {
                let mut state = state.borrow_mut();

                // A connection might be added to the set when the user either activates the
                // connection switch or connection is added from the disk at startup. We ignore
                // the second case here, since the connection will already be loaded at the state.
                // if state.conns.iter().find(|c| c.is_like(&conn) ).is_none() {
                state.conns.push(conn);
                // }

            }
        });
    }

}

impl React<super::OpenedScripts> for SharedUserState {

    fn react(&self, scripts : &OpenedScripts) {
        scripts.connect_file_persisted({
            let state = self.clone();
            move |file| {
                add_file(&state, file);
            }
        });
        scripts.connect_opened({
            let state = self.clone();
            move |file| {
                add_file(&state, file);
            }
        });
        scripts.connect_added({
            let state = self.clone();
            move |file| {
                add_file(&state, file);
            }
        });
    }

}*/

/*fn add_file(state : &SharedUserState, file : OpenedFile) {
    let mut state = state.borrow_mut();
    if let Some(path) = &file.path {
        if state.scripts.iter().find(|f| &f.path.as_ref().unwrap()[..] == &path[..] ).is_none() {
            state.scripts.push(file);
        }
    }
}*/

impl React<crate::ui::QueriesWindow> for SharedUserState {

    fn react(&self, win : &QueriesWindow) {
        let state = self.clone();
        let main_paned = win.paned.clone();
        let sidebar_paned = win.sidebar.paned.clone();
        win.window.connect_close_request(move |win| {
            // Query all paned positions
            let main_paned_pos = main_paned.position();
            let side_paned_pos = sidebar_paned.position();
            {
                let mut state = state.borrow_mut();
                state.main_handle_pos = main_paned_pos;
                state.side_handle_pos = side_paned_pos;
                state.window_width = win.allocation().width;
                state.window_height = win.allocation().height;
            }
            gtk4::Inhibit(false)
        });

        win.settings.report_bx.entry.connect_changed({
            let state = self.clone();
            move|entry| {
                let path = entry.text().as_str().to_string();
                if !path.is_empty() {
                    let mut state = state.borrow_mut();
                    state.templates.clear();
                    state.templates.push(path);
                }
            }
        });
        win.settings.security_bx.cert_added.connect_activate({
            let state = self.clone();
            move |_, param| {
                if let Some(s) = param {
                    let cert : Certificate = serde_json::from_str(&s.get::<String>().unwrap()).unwrap();
                    let mut state = state.borrow_mut();

                    let mut updated = false;
                    while let Some(ix) = state.conns.iter().position(|c| c.host == cert.host ) {
                        state.conns[ix].cert = Some(cert.cert.clone());
                        updated = true;
                    }

                    if !updated {
                        state.unmatched_certs.push(cert);
                    }

                }
            }
        });
        win.settings.security_bx.cert_removed.connect_activate({
            let state = self.clone();
            move |_, param| {
                if let Some(s) = param {
                    let cert : Certificate = serde_json::from_str(&s.get::<String>().unwrap()).unwrap();
                    let mut state = state.borrow_mut();
                    while let Some(ix) = state.conns.iter().position(|c| c.host == cert.host ) {
                        state.conns[ix].cert = None;
                    }
                }
            }
        });
    }

}

pub fn set_window_state(user_state : &SharedUserState, queries_win : &QueriesWindow) {
    let state = user_state.borrow();
    queries_win.paned.set_position(state.main_handle_pos);
    queries_win.sidebar.paned.set_position(state.side_handle_pos);
    queries_win.window.set_default_size(state.window_width, state.window_height);

    for conn in state.conns.iter() {
        if let Some(cert) = conn.cert.as_ref() {
            crate::ui::append_certificate_row(
                queries_win.settings.security_bx.exp_row.clone(),
                &conn.host,
                cert,
                &queries_win.settings.security_bx.rows
            );
        }
    }
}

pub fn set_client_state(user_state : &SharedUserState, client : &QueriesClient) {
    let state = user_state.borrow();
    client.conn_set.add_connections(&state.conns);
    client.scripts.add_files(&state.scripts);
    crate::log_debug_if_required("Client updated with user state");
}

// React to all common data structures, to persist state to filesystem.
// impl React<ActiveConnection> for UserState { }

