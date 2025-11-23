use std::rc::Rc;

mod models;

mod db;
use db::*;

mod ui;

mod io_utils;
use io_utils::*;

mod navigator;
use navigator::*;

fn main() {
    let db = Rc::new(JiraDatabase::new("data/db.json".to_owned()));
    let mut nav = Navigator::new(db);
    
    loop {
        clearscreen::clear().unwrap();

        let page = nav.get_current_page();

        let page = match page {
            Some(page) => page,
            None => {
                let _ = nav.handle_action(models::Action::Exit);
                break
            }
        };

        let _ = page.draw_page();
        let input = get_user_input();

        let action = page.handle_input(&input);
        
        let _ = match action {
            Ok(action) => {
                match action {
                    Some(action) => nav.handle_action(action),
                    None => continue
                }
                
            },
            Err(_) => continue
        };
    }
}