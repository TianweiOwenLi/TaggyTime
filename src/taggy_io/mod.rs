
use crate::MinInstant;
use crate::Tasks;


pub struct record {
    content: Vec<datapoint>, 
    todo: std::collections::BTreeSet<Tasks>,
}

#[derive(Clone)]
struct datapoint (MinInstant, Tasks);

impl datapoint {
    pub fn now(offset: i32, t: Tasks) -> Self {
        Self(MinInstant::now(offset), t)
    }



}

impl record { // TODO add past statistics method

    /// create a new empty record
    pub fn empty() -> Self {
        record { content: vec![], todo: std::collections::BTreeSet::new() }
    }

    /// read an existing record from file
    pub fn read_from_file() -> Self {
        todo!()
    }

    /// returns a cloned datapoint from record 
    pub fn top_datapoint(&self) -> Option<datapoint> {
        self.content.last().cloned()
    }

    /// pushes a datapoint
    pub fn log_datapoint(&mut self, offset: i32, t: Tasks) {
        self.content.push(datapoint::now(offset, t));
    }

    /// add items into todo list
    pub fn add_todo(&mut self, t: Tasks) {
        let x = match t { 
            Tasks::Classwork(_) => t,
            Tasks::Projects(_) => t,
            Tasks::Logistic(_) => t,
            _ => panic!("cannot add this to todo"),
        };

        self.todo.insert(x);
    }

    /// remove items from todo list
    pub fn remove_todo(&mut self, t: Tasks) {
        self.todo.remove(&t);
    }
    
}
