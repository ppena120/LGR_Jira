use anyhow::{Result, anyhow};
use serde_json;
use std::{collections::HashMap, fs};

use crate::models::{DBState, Epic, Story, Status};

pub struct JiraDatabase {
    pub database: Box<dyn Database>
}

impl JiraDatabase {
    pub fn new(file_path: String) -> Self {
        Self {
            database: Box::new(JSONFileDatabase{file_path})
        }
    }

    pub fn read_db(&self) -> Result<DBState> {
        let db_state = self.database.read_db()?;

        Ok(db_state)
    }
    
    pub fn create_epic(&self, epic: Epic) -> Result<u32> {
        let mut db_state = self.read_db()?;

        db_state.last_item_id += 1;
        db_state.epics.insert(db_state.last_item_id, epic);

        self.database.write_db(&db_state)?;

        Ok(db_state.last_item_id)
    }
    
    pub fn create_story(&self, story: Story, epic_id: u32) -> Result<u32> {
        let mut db_state = self.read_db()?;

        db_state.last_item_id += 1;
        db_state.stories.insert(db_state.last_item_id, story);

        db_state.epics.get_mut(&epic_id)
            .ok_or(anyhow!("Story Creation Error: Invalid epic_id"))?
            .stories.push(db_state.last_item_id);

        self.database.write_db(&db_state)?;

        Ok(db_state.last_item_id)
    }
    
    pub fn delete_epic(&self, epic_id: u32) -> Result<()> {
        let mut db_state = self.read_db()?;

        let story_ids_in_epic = &db_state.epics.get(&epic_id)
            .ok_or(anyhow!("Epic Deletion Error: Invalid epic_id - Could not find epic_id"))?
            .stories;

        for story_id in story_ids_in_epic {
            db_state.stories.remove(&*story_id)
                .ok_or(anyhow!("Epic Deletion Error: Failed during stories deletion"))?;
        }

        db_state.epics.remove(&epic_id)
            .ok_or(anyhow!("Epic Deletion Error: Invalid epic_id - Could not remove epic_id"))?;

        self.database.write_db(&db_state)?;

        Ok(())
    }
    
    pub fn delete_story(&self,epic_id: u32, story_id: u32) -> Result<()> {
        let mut db_state = self.read_db()?;

        // Check if Story is in Epic
        let story_is_in_epic = db_state.epics.get(&epic_id)
            .ok_or(anyhow!("Story Deletion Error: Invalid epic_id - Could not find epic_id"))?
            .stories
            .contains(&story_id);

        if !story_is_in_epic {
            return Err(anyhow!("Story Deletion Error: Invalid epic_id - Could not find epic_id"));
        }

        // Delete Story from Epic
        db_state.epics.get_mut(&epic_id)
            .ok_or(anyhow!("Story Deletion Error: Invalid epic_id - Could not find epic_id"))?
            .stories.pop_if(|x| *x == story_id);

        // Delete Story from Stories
        db_state.stories.remove(&story_id)
            .ok_or(anyhow!("Story Deletion Error: Invalid epic_id - Could not find story_id"))?;

        // Write database
        self.database.write_db(&db_state)?;

        Ok(())
    }
    
    pub fn update_epic_status(&self, epic_id: u32, status: Status) -> Result<()> {
        let mut db_state = self.read_db()?;

        // Update Epic status
        db_state.epics.get_mut(&epic_id)
            .ok_or(anyhow!("Update Epic Error: Invalid epic_id - Could not find epic_id: {epic_id}"))?
            .status = status;

        // Write database
        self.database.write_db(&db_state)?;

        Ok(())
    }
    
    pub fn update_story_status(&self, story_id: u32, status: Status) -> Result<()> {
        let mut db_state = self.read_db()?;

        // Update Story status
        db_state.stories.get_mut(&story_id)
            .ok_or(anyhow!("Update Story Error: Invalid story_id - Could not find story_id: {story_id}"))?
            .status = status;

        // Write database
        self.database.write_db(&db_state)?;

        Ok(())
    }
}

pub trait Database {
    fn read_db(&self) -> Result<DBState>;
    fn write_db(&self, db_state: &DBState) -> Result<()>;
}

struct JSONFileDatabase {
    pub file_path: String
}

impl Database for JSONFileDatabase {
    fn read_db(&self) -> Result<DBState> {
        let json_data = fs::read_to_string(&self.file_path)?;
        let json_data: DBState = serde_json::from_str(&json_data)?;

        Ok(json_data)
    }

    fn write_db(&self, db_state: &DBState) -> Result<()> {
        let json_data = serde_json::to_string_pretty(&db_state)?;
        fs::write(&self.file_path, json_data)?;

        Ok(())
    }
}

pub mod test_utils {
    use std::{cell::RefCell, collections::HashMap};

    use super::*;
    
    pub struct MockDB {
        last_written_state: RefCell<DBState>
    }

    impl MockDB {
        pub fn new() -> Self {
            Self { last_written_state: RefCell::new(DBState { last_item_id: 0, epics: HashMap::new(), stories: HashMap::new() }) }
        }    
    }

    impl Database for MockDB {
        fn read_db(&self) -> Result<DBState> {
            let state = self.last_written_state.borrow().clone();
            Ok(state)
        }

        fn write_db(&self, db_state: &DBState) -> Result<()> {
            let latest_state = &self.last_written_state;
            *latest_state.borrow_mut() = db_state.clone();
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::MockDB;

    #[test]
    fn create_epic_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic.clone());
        
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 1;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&id), Some(&epic));
    }

    #[test]
    fn create_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let story = Story::new("".to_owned(), "".to_owned());

        let non_existent_epic_id = 999;

        let result = db.create_story(story, non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn create_story_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story.clone(), epic_id);
        assert_eq!(result.is_ok(), true);

        let id = result.unwrap();
        let db_state = db.read_db().unwrap();

        let expected_id = 2;

        assert_eq!(id, expected_id);
        assert_eq!(db_state.last_item_id, expected_id);
        assert_eq!(db_state.epics.get(&epic_id).unwrap().stories.contains(&id), true);
        assert_eq!(db_state.stories.get(&id), Some(&story));
    }

    #[test]
    fn delete_epic_should_error_if_invalid_epic_id() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };

        let non_existent_epic_id = 999;

        let result = db.delete_epic(non_existent_epic_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_epic_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_epic(epic_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        println!("db_state.epics.get(&epic_id): {:?}", db_state.epics.get(&epic_id));
        assert_eq!(db_state.epics.get(&epic_id), None);
        println!("db_state.stories.get(&story_id): {:?}", db_state.stories.get(&story_id));
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn delete_story_should_error_if_invalid_epic_id() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);
        
        let story_id = result.unwrap();

        let non_existent_epic_id = 999;
        
        let result = db.delete_story(non_existent_epic_id, story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_error_if_story_not_found_in_epic() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let non_existent_story_id = 999;
        
        let result = db.delete_story(epic_id, non_existent_story_id);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn delete_story_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);
        assert_eq!(result.is_ok(), true);

        let story_id = result.unwrap();

        let result = db.delete_story(epic_id, story_id);
        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        let expected_last_id = 2;

        assert_eq!(db_state.last_item_id, expected_last_id);
        assert_eq!(db_state.epics.get(&epic_id).unwrap().stories.contains(&story_id), false);
        assert_eq!(db_state.stories.get(&story_id), None);
    }

    #[test]
    fn update_epic_status_should_error_if_invalid_epic_id() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };

        let non_existent_epic_id = 999;

        let result = db.update_epic_status(non_existent_epic_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_epic_status_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);
        
        assert_eq!(result.is_ok(), true);

        let epic_id = result.unwrap();

        let result = db.update_epic_status(epic_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.epics.get(&epic_id).unwrap().status, Status::Closed);
    }

    #[test]
    fn update_story_status_should_error_if_invalid_story_id() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };

        let non_existent_story_id = 999;

        let result = db.update_story_status(non_existent_story_id, Status::Closed);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn update_story_status_should_work() {
        let db = JiraDatabase { database: Box::new(MockDB::new()) };
        let epic = Epic::new("".to_owned(), "".to_owned());
        let story = Story::new("".to_owned(), "".to_owned());

        let result = db.create_epic(epic);

        let epic_id = result.unwrap();

        let result = db.create_story(story, epic_id);

        let story_id = result.unwrap();

        let result = db.update_story_status(story_id, Status::Closed);

        assert_eq!(result.is_ok(), true);

        let db_state = db.read_db().unwrap();

        assert_eq!(db_state.stories.get(&story_id).unwrap().status, Status::Closed);
    }

    mod database {
        use std::collections::HashMap;
        use std::io::Write;

        use super::*;

        #[test]
        fn read_db_should_fail_with_invalid_path() {
            let db = JSONFileDatabase { file_path: "INVALID_PATH".to_owned() };
            assert_eq!(db.read_db().is_err(), true);
        }

        #[test]
        fn read_db_should_fail_with_invalid_json() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0 epics: {} stories {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_err(), true);
        }

        #[test]
        fn read_db_should_parse_json_file() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

            let result = db.read_db();

            assert_eq!(result.is_ok(), true);
        }

        #[test]
        fn write_db_should_work() {
            let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

            let file_contents = r#"{ "last_item_id": 0, "epics": {}, "stories": {} }"#;
            write!(tmpfile, "{}", file_contents).unwrap();

            let db = JSONFileDatabase { file_path: tmpfile.path().to_str()
                .expect("failed to convert tmpfile path to str").to_string() };

            let story = Story { name: "epic 1".to_owned(), description: "epic 1".to_owned(), status: Status::Open };
            let epic = Epic { name: "epic 1".to_owned(), description: "epic 1".to_owned(), status: Status::Open, stories: vec![2] };

            let mut stories = HashMap::new();
            stories.insert(2, story);

            let mut epics = HashMap::new();
            epics.insert(1, epic);

            let state = DBState { last_item_id: 2, epics, stories };

            let write_result = db.write_db(&state);
            let read_result = db.read_db().unwrap();

            assert_eq!(write_result.is_ok(), true);
            assert_eq!(read_result, state);
        }
    }
}