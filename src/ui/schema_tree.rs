use gtk4::*;
use gtk4::prelude::*;
use gio::prelude::*;
use std::rc::Rc;
use std::cell::{RefCell};
use std::fs::File;
use std::io::Read;
use crate::tables::environment::{TableEnvironment, EnvironmentUpdate};
use crate::sql::object::{DBObject, DBType, DBInfo};
// use sourceview::*;
// use gtk::prelude::*;
// use crate::{status_stack::StatusStack};
// use crate::status_stack::*;
// use crate::utils;
// use sourceview::View;
// use crate::editor::SqlEditor;
use std::path::{Path, PathBuf};
use glib::{types::Type, value::{Value, ToValue}};
use gdk_pixbuf::Pixbuf;
use std::collections::HashMap;
use gdk::{RGBA, EventType};
// use crate::editor::{QuerySchedule, ExecutionState};
use std::env;
use crate::React;
use crate::client::ActiveConnection;
use crate::ui::PackedImageLabel;

/*pub trait View {

    type Model;

    /// This makes the widget wrap the object into a Rc<RefCell<Model>>
    fn build(m : Self::Model);

    fn share(&self) -> Rc<RefCell<Self::Model>>;

    /// This makes the widget borrow the model
    fn view(&self, impl Fn(&self, &Self::Model));

    /// This makes the widget borrow the model mutably.
    fn update(&self, impl Fn(&self, &mut Self::Model));
}

*/
//use either::Either;

/*/// Implemented by types which can be viewed by modifying the given widget
/// (assumed to be a wrapped pointer)
pub trait Show<W>
where
    W : WidgetExt
{
    fn show(&self, wid : &W);
}

impl Show<Image> for DBObject {

    fn show(&self, wid : &Image) {
        match self {
            DBObject::Schema{ name, children } => {

            },
            DBObject::Table{ name, cols } => {

            }
        }
    }

}

impl Show<Label> for DBObject {

    fn show(&self, wid : &Label) {
        match self {
            DBObject::Schema{ name, children } => {

            },
            DBObject::Table{ name, cols } => {

            }
        }
    }
}*/

pub enum Growth<T> {
    Depth(T),
    Breadth(T),
    Halt
}

// IconTree<T : Display + Iterator<Item=Growth<&Self, &Self>>>
// icontree::build(.) then takes a HashMap<String, Pixbuf> at its initialization.
// matching left grows the tree in a depth fashion; matching right rows the
// tree in a breadth fashion.
#[derive(Clone, Debug)]
pub struct SchemaTree {
    tree_view : TreeView,
    model : TreeStore,

    type_icons : Rc<HashMap<DBType, Pixbuf>>,

    tbl_icon : Pixbuf,
    clock_icon : Pixbuf,
    schema_icon : Pixbuf,
    fn_icon : Pixbuf,
    view_icon : Pixbuf,
    key_icon : Pixbuf,

    // db_objs : Rc<RefCell<Option<Vec<DBObject>>>>,

    // selected_obj : Rc<RefCell<Option<(DBObject, Vec<i32>)>>>,

    // pub schema_popover : SchemaPopover

    schema_popover : PopoverMenu,

    scroll : ScrolledWindow,

    pub bx : Box
}

const ALL_TYPES : [DBType; 15] = [
    DBType::Bool,
    DBType::I16,
    DBType::I32,
    DBType::I64,
    DBType::F32,
    DBType::F64,
    DBType::Numeric,
    DBType::Text,
    DBType::Date,
    DBType::Time,
    DBType::Bytes,
    DBType::Json,
    DBType::Xml,
    DBType::Array,
    DBType::Unknown
];

/*#[derive(Debug, Clone)]
pub struct SchemaPopover {
    query_btn : ModelButton,
    import_btn : ModelButton,
    model_btn : ModelButton,
    listen_btn : ModelButton,
    insert_btn : ModelButton,
    table_menu : PopoverMenu,
    schema_menu : PopoverMenu
}

impl SchemaPopover {

    fn build(builder : &Builder) -> Self {
        let query_btn : ModelButton = builder.get_object("query_model_btn").unwrap();
        let import_btn : ModelButton = builder.get_object("import_btn").unwrap();
        let model_btn : ModelButton = builder.get_object("model_btn").unwrap();
        let listen_btn : ModelButton = builder.get_object("listen_table_btn").unwrap();
        let insert_btn : ModelButton = builder.get_object("insert_btn").unwrap();
        let schema_menu : PopoverMenu = builder.get_object("schema_menu").unwrap();
        let table_menu : PopoverMenu = builder.get_object("table_menu").unwrap();
        Self { query_btn, import_btn, model_btn, listen_btn, insert_btn, table_menu, schema_menu }
    }

    /*fn connect(&self, sql_editor : &SqlEditor) {

    }*/

}*/

impl SchemaTree {

    pub fn build() -> Self {
        let type_icons = load_type_icons();
        let tbl_icon = Pixbuf::from_file_at_scale(&icon_path("queries-symbolic.svg").unwrap(), 16, 16, true).unwrap();
        // let tbl_image = Image::from_icon_name(Some("grid-black"), IconSize::SmallToolbar);
        // let tbl_icon = tbl_image.get_pixbuf().unwrap();

        // let theme = IconTheme::default();
        // println!("{:?}", theme.load_icon("grid-black", 16, IconLookupFlags::empty()));

        /*let info = IconInfo::new_for_pixbuf(&theme, &tbl_icon);
        let tbl_icon = match info.load_symbolic(&RGBA::white(), None, None, None) {
             Ok(icon) => if icon.1 {
                icon.0
            } else {
                panic!("Loaded icon was not symbolic");
            },
            Err(e) => panic!("Symbolic icon load error: {:?}", e)
        };*/

        // let db_objs : Rc<RefCell<Option<Vec<DBObject>>>> = Rc::new(RefCell::new(None));
        // let selected_obj : Rc<RefCell<Option<(DBObject, Vec<i32>)>>> = Rc::new(RefCell::new(None));

        // let schema_popover = SchemaPopover::build(&builder);
        let menu = gio::Menu::new();
        menu.append(Some("Query"), Some("win.query"));
        menu.append(Some("Insert"), Some("win.insert"));
        let schema_popover = PopoverMenu::from_model(Some(&menu));

        // table_menu.upcast::<Popover>().set_transition_type(RevealerTransitionType::SlideRight);
        // schema_menu.upcast::<Popover>().set_transition_type(RevealerTransitionType::SlideRight);

        let schema_icon = Pixbuf::from_file_at_scale(&icon_path("db.svg").unwrap(), 16, 16, true).unwrap();
        let fn_icon = Pixbuf::from_file_at_scale(&icon_path("fn-dark.svg").unwrap(), 16, 16, true).unwrap();
        let clock_icon = Pixbuf::from_file_at_scale(&icon_path("clock-app-symbolic.svg").unwrap(), 16, 16, true).unwrap();
        let view_icon = Pixbuf::from_file_at_scale(&icon_path("view.svg").unwrap(), 16, 16, true).unwrap();
        let key_icon = Pixbuf::from_file_at_scale(&icon_path("key-symbolic.svg").unwrap(), 16, 16, true).unwrap();

        // let tree_view : TreeView = builder.get_object("schema_tree_view").unwrap();
        let tree_view = TreeView::builder().valign(Align::Fill).vexpand(true).build();

        let model = configure_tree_view(&tree_view);

        // tree_view.get_background_area(None, None).connect_clicked(move |_| {
        //    println!("Clicked background");
        // });

        /*for col in tree_view.get_columns() {
            col.connect_clicked(move |_| {
                println!("Clicked column");
            });
        }*/

        let title = PackedImageLabel::build("db-symbolic", "Schema");
        title.bx.set_vexpand(false);
        title.bx.set_valign(Align::Start);
        super::set_border_to_title(&title.bx);
        let bx = Box::new(Orientation::Vertical, 0);

        let scroll = ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_valign(Align::Fill);
        scroll.set_child(Some(&tree_view));
        bx.append(&title.bx);
        bx.append(&scroll);

        Self{ tree_view, model, type_icons, tbl_icon, schema_icon, fn_icon, clock_icon, view_icon, key_icon, schema_popover, bx, scroll }
    }

    // grow_tree<T>(obj : T) for T : Display + Iterator<Item=&Self>
    // and receive a HashMap<&str, Pixbuf> which maps the Display key to a Pixbuf living at this hash.
    fn grow_tree(&self, model : &TreeStore, parent : Option<&TreeIter>, obj : DBObject) {
        match obj {
            DBObject::Schema{ name, children } => {
                // println!("Adding schema {:?} to model", name);
                let schema_pos = model.append(parent);
                model.set(&schema_pos, &[(0, &self.schema_icon), (1, &name)]);
                for child in children {
                    self.grow_tree(&model, Some(&schema_pos), child);
                }
            },
            DBObject::Table{ name, cols, rels, .. } => {
                // println!("Adding table {:?} to model", name);
                // println!("Adding columns {:?} to model", cols);
                let tbl_pos = model.append(parent);
                model.set(&tbl_pos, &[(0, &self.tbl_icon), (1, &name.to_value())]);
                for c in cols {
                    let col_pos = model.append(Some(&tbl_pos));
                    let is_pk = c.2;
                    let opt_rel = rels.iter().find(|rel| &rel.src_col[..] == &c.0[..] );
                    let is_fk = opt_rel.is_some();
                    let name : String = if let Some(rel) = opt_rel {
                        let tgt_schema = if &rel.tgt_schema[..] == "public" {
                            format!("")
                        } else {
                            format!("{}.", rel.tgt_schema)
                        };
                        format!("{} ({}{})", c.0, tgt_schema, rel.tgt_tbl )
                    } else {
                        format!("{}", c.0)
                    };
                    let icon = if is_fk || is_pk {
                        &self.key_icon
                    } else {
                        &self.type_icons[&c.1]
                    };
                    model.set(&col_pos, &[(0, icon), (1, &name.to_value())]);
                }
            },
            DBObject::Function { name, args, ret } => {
                let schema_pos = model.append(parent);
                let args_str = args.iter().map(|a| a.to_string() ).collect::<Vec<_>>().join(",");
                let sig = format!("{}({}) {}", name, args_str, ret );
                model.set(&schema_pos, &[(0, &self.fn_icon.to_value()), (1, &sig.to_value())]);
            },
            DBObject::View { name } => {
                let schema_pos = model.append(parent);
                model.set(&schema_pos, &[(0, &self.view_icon.to_value()), (1, &name.to_value())]);
            }
        }
    }

    pub fn repopulate(&self, objs : Vec<DBObject>) {
        self.model.clear();
        let mut is_pg = false;
        for obj in objs {
            self.grow_tree(&self.model, None, obj);
        }
        self.model.foreach(|model, path, iter| {
            if path.depth() == 1 {
                self.tree_view.expand_row(path, false);
            }
            false
        });
    }

    pub fn clear(&self) {
        self.model.clear();
        // self.tree_view.show_all();
    }

    /*pub fn connect(&self, sql_editor : &SqlEditor, t_env : &Rc<RefCell<TableEnvironment>>) {
        let model = self.model.clone();

        self.tree_view.connect_button_press_event({
            let table_menu = self.schema_popover.table_menu.clone();
            let schema_menu = self.schema_popover.schema_menu.clone();
            let db_objs = self.db_objs.clone();
            let selected_obj = self.selected_obj.clone();
            let clock = sql_editor.update_clock.clone();
            let listen_btn = self.schema_popover.listen_btn.clone();
            move |view, ev_btn| {
                if ev_btn.get_event_type() == EventType::ButtonPress && ev_btn.get_button() == 3 {
                    let (x, y) = if let Some((y, x)) = ev_btn.get_coords() {
                        (y, x)
                    } else {
                        return glib::signal::Inhibit(false);
                    };
                    let opt_path = view.get_path_at_pos(x as i32, y as i32);
                    if let Some((opt_path, opt_col, _, _)) = opt_path {
                        if let Some(path) = &opt_path {
                            let res_ixs : Result<Vec<usize>, ()> = path.get_indices()
                                .iter()
                                .map(|ix| if *ix >= 0 { Ok(*ix as usize) } else { Err(()) })
                                .collect();
                            if let Ok(ixs) = res_ixs {
                                if ixs.len() >= 1 {
                                    if let Some(objs) = &*db_objs.borrow() {
                                        let opt_objs : Option<DBObject> = objs.get(ixs[0]).cloned();
                                        if let Some(root_obj) = opt_objs {
                                            let obj = if ixs.len() == 1 {
                                                Some(root_obj)
                                            } else {
                                                root_obj.get_table_or_schema(&ixs[1..])
                                            };
                                            if let Some(obj) = obj {
                                                println!("Selected {:?}", obj);
                                                let area = view.get_cell_area(opt_path.as_ref(), opt_col.as_ref());
                                                *(selected_obj.borrow_mut()) = Some((obj.clone(), path.get_indices()));
                                                let menu = match &obj {
                                                    DBObject::Table { .. } => {
                                                        &table_menu
                                                    },
                                                    DBObject::Schema { .. } => {
                                                        &schema_menu
                                                    },
                                                    _ => {
                                                        &table_menu
                                                    }
                                                };

                                                // });
                                                match clock.borrow().clone() {
                                                    QuerySchedule::Notification { selection, .. } => {
                                                        if &selection[..] == &path.get_indices()[..] {
                                                            listen_btn.set_property_text(Some("Unlisten"));
                                                            listen_btn.set_sensitive(true);
                                                        } else {
                                                            listen_btn.set_property_text(Some("Listen"));
                                                            listen_btn.set_sensitive(false);
                                                        }
                                                    },
                                                    _ => {
                                                        listen_btn.set_property_text(Some("Listen"));
                                                        listen_btn.set_sensitive(true);
                                                    }
                                                }

                                                menu.set_relative_to(Some(view));
                                                menu.set_pointing_to(&area);
                                                menu.show();

                                            } else {
                                                println!("No table or schema object selected");
                                            }
                                        }
                                    } else {
                                        println!("Unable to borrow db objects");
                                    }
                                } else {
                                    println!("Tree iter did not yield indices");
                                }
                            }
                        }
                    } else {
                        println!("Clicked at empty location");
                    }
                }

                glib::signal::Inhibit(false)
            }
        });

        self.schema_popover.query_btn.connect_clicked({
            let selected_obj = self.selected_obj.clone();
            let sql_editor = sql_editor.clone();
            let t_env = t_env.clone();
            move |btn| {
                println!("Query clicked");
                if let Some((obj, _)) = &*selected_obj.borrow() {
                    match &obj {
                        DBObject::Table { name, .. } => {
                            if let Ok(mut env) = t_env.try_borrow_mut() {
                                env.prepare_and_send_query(format!("select * from {} limit 500;", name), HashMap::new(), true).unwrap();
                                *sql_editor.query_sent.borrow_mut() = ExecutionState::Evaluating;
                            } else {
                                println!("Unable to borrow table environment");
                            }
                        },
                        _ => { }
                    }
                }
            }
        });

        self.schema_popover.import_btn.connect_clicked(move |btn| {
            // Open the CSV import dialog
        });
        self.schema_popover.model_btn.connect_clicked(move |btn| {
            // Open SVG as a new sheet tab by rendering it into a drawarea using rsvg.
            // The "Export" button now sets the type as SVG. The Global menu should also
            // have a "generate" option that saves a file with the SQL content to either:
            // Query a table (if table output, maybe filtered or selected)
            // Create a full schema or table (if the model is selected)
            //
        });
        self.schema_popover.listen_btn.connect_clicked({
            let selected_obj = self.selected_obj.clone();
            let clock = sql_editor.update_clock.clone();
            let model = self.model.clone();
            let tbl_icon = self.tbl_icon.clone();
            let clock_icon = self.clock_icon.clone();
            move |btn| {

                if let Some((obj, sel_ixs)) = &*selected_obj.borrow() {

                    match &obj {
                        DBObject::Table { name, .. } => {
                            let mut schedule = clock.borrow_mut();
                            match &schedule.clone() {
                                QuerySchedule::Notification { .. } => {
                                    *schedule = QuerySchedule::Off;

                                    model.foreach(|_, path, iter| {
                                        if &path.get_indices()[..] == &sel_ixs[..] {
                                            model.set(&iter, &[0, 1], &[&tbl_icon.to_value(), &(&name).to_value()]);
                                            true
                                        } else {
                                            false
                                        }
                                    });

                                    btn.set_property_text(Some("Unlisten"));
                                },
                                QuerySchedule::Off | QuerySchedule::Interval { ..} => {
                                    *schedule = QuerySchedule::Notification {
                                        channel : format!("inserts"),
                                        filter : format!("{{ \"table\" : \"{}\" }}", name),
                                        selection : sel_ixs.clone()
                                    };

                                    // Also consider model.selected_foreach
                                    model.foreach(|_, path, iter| {
                                        if &path.get_indices()[..] == &sel_ixs[..] {
                                            //
                                            model.set(&iter, &[0, 1], &[&clock_icon.to_value(), &(&name).to_value()]);
                                            true
                                        } else {
                                            false
                                        }
                                    });
                                    btn.set_property_text(Some("Listen"));
                                }
                            }
                        },
                        _ => {

                        }
                    }
                }

                // (1) Retrieve table name
                // (2) Change the update clock with the table name and the desired actions
                // The desired actions (whether insert, update or delete) are set at the
                // settings menu (Listen for: Inserts, Updates, Deletes).
            }
        });
        self.schema_popover.insert_btn.connect_clicked(move |btn| {
            // (1) Extract table model
            // (2) Build form for the table model (excluding primary keys)
            // (3) Create SQL statement any time the insert button below the form is clicked.
            // (4) Either click insert to execute the SQL or generate to save a file with
            // the desired SQL content.
        });

       /*// TODO must guarantee treeview::button_press is always called before this.
       self.schema_popover.table_menu.connect_show({
            let selected_obj = self.selected_obj.clone();
            let tree_view = self.tree_view.clone();
            let listen_btn = self.schema_popover.listen_btn.clone();
            let clock = sql_editor.update_clock.clone();
            move |_| {

                let mut this_selected = false;
                tree_view.get_selection().selected_foreach(|_, path, iter| {
                    if let Some(sel) = selected_obj.borrow().as_ref().map(|sel| sel.1.clone() ) {
                        if &sel[..] == &path.get_indices()[..] {
                            this_selected = true;
                            println!("This selected at {:?}", sel);
                        }
                    }
                });

                match clock.borrow().clone() {
                    QuerySchedule::Off => {
                        listen_btn.set_property_text(Some("Listen"));
                        listen_btn.set_sensitive(true);
                    },
                    _ => {
                        if this_selected {
                            listen_btn.set_property_text(Some("Unlisten"));
                            listen_btn.set_sensitive(true);
                        } else {
                            listen_btn.set_property_text(Some("Listen"));
                            listen_btn.set_sensitive(false);
                        }
                    }
                }
            }
        });*/
    }*/

}

impl React<ActiveConnection> for SchemaTree {

    fn react(&self, conn : &ActiveConnection) {
        let schema_tree = self.clone();
        conn.connect_db_connected(move |info : Option<DBInfo>| {
            if let Some(info) = info {
                schema_tree.repopulate(info.schema);
            }
        });
    }

}

fn load_type_icons() -> Rc<HashMap<DBType, Pixbuf>> {
    let mut type_icons = HashMap::new();
    for ty in ALL_TYPES.iter() {
        let path = match ty {
            DBType::Bool => "boolean.svg",
            DBType::I16 | DBType::I32 | DBType::I64 => "integer.svg",
            DBType::F32 | DBType::F64 | DBType::Numeric => "real.svg",
            DBType::Text => "text.svg",
            DBType::Date => "date.svg",
            DBType::Time => "time.svg",
            DBType::Json => "json.svg",
            DBType::Xml => "xml.svg",
            DBType::Bytes => "binary.svg",
            DBType::Array => "array.svg",
            DBType::Unknown => "unknown.svg",
            DBType::Trigger => "unknown.svg"
        };
        let pix = Pixbuf::from_file_at_scale(&icon_path(&path).unwrap(), 16, 16, true).unwrap();
        type_icons.insert(*ty, pix);
    }
    Rc::new(type_icons)
}

fn configure_tree_view(tree_view : &TreeView) -> TreeStore {
    let model = TreeStore::new(&[Pixbuf::static_type(), Type::STRING]);
    tree_view.set_model(Some(&model));
    let pix_renderer = CellRendererPixbuf::new();
    pix_renderer.set_padding(6, 6);
    // pix_renderer.set_property_height(24);
    let txt_renderer = CellRendererText::new();
    // txt_renderer.set_property_height(24);

    let pix_col = TreeViewColumn::new();
    pix_col.pack_start(&pix_renderer, false);
    pix_col.add_attribute(&pix_renderer, "pixbuf", 0);

    let txt_col = TreeViewColumn::new();
    txt_col.pack_start(&txt_renderer, true);
    txt_col.add_attribute(&txt_renderer, "text", 1);

    tree_view.append_column(&pix_col);
    tree_view.append_column(&txt_col);
    tree_view.set_show_expanders(true);
    tree_view.set_can_focus(false);
    tree_view.set_has_tooltip(false);
    tree_view.set_headers_visible(false);

    // tree_view.set_vadjustment(Some(&Adjustment::default()));
    // tree_view.set_vadjustment(Some(&Adjustment::new(0.0, 0.0, 100.0, 10.0, 10.0, 100.0)));
    // tree_view.set_vscroll_policy(ScrollablePolicy::Natural);

    model
}

fn icon_path(filename : &str) -> Result<String, &'static str> {
    let exe_dir = exec_dir()?;
    let path = exe_dir + "/../../assets/icons/" + filename;
    Ok(path)
}

fn exec_dir() -> Result<String, &'static str> {
    let exe_path = env::current_exe().map_err(|_| "Could not get executable path")?;
    let exe_dir = exe_path.as_path().parent().ok_or("CLI executable has no parent dir")?
        .to_str().ok_or("Could not convert path to str")?;
    Ok(exe_dir.to_string())
}
