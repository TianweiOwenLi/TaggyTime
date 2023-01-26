use std::ops::Index;
use crate::command_parser::Command;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tasks {            // SPECIAL is totally not an allusion to Fallout
    Sleep,                  // naps do count
    Projects(String),       // extracurriculars + learning
    Exercise,               // physical exercise
    Classwork(String),      // only the mandatory ones
    Interaction,            // interactions with humans
    Activity,               // fun!
    Logistic(String),       // anything else + meta
}

// fn starts_ends_with()


impl Tasks {
    pub fn from_str(task_str: String) -> Option<Self> {
        let parsed = Command::parse(task_str);
        match parsed {

            // parsing tasks that does not contain a string
            Some(Command::Atomic(s)) => match s.as_str() {
                "Sleep" => Some(Self::Sleep),
                "Exercise" => Some(Self::Exercise),
                "Interaction" => Some(Self::Interaction),
                "Activity" => Some(Self::Activity),
                _ => None
            }, 

            // parsing tasks that contains a string
            // Some(Command::Tag(s, v)) => {
            //     if v.len() == 1 {
            //         let cmd_in_v = 
            //         v.get(0).expect("v[0] should be accessible");
                    
            //         let contained_string = match cmd_in_v {
            //             Command::
            //         }

            //          match s.as_str() {
            //             "Projects" => Some(Self::Projects(contained_string)),
            //             "Classwork" => Some(Self::Classwork(v
            //                 .get(0)
            //                 .expect("v should have len=1")
            //             )),
            //             "Logistic" => Some(Self::Logistic(v
            //                 .get(0)
            //                 .expect("v should have len=1")
            //             )),
            //             _ => None,

            //         }
            //     } else {
            //         None
            //     } // mismatched arity
                
            // },

            // command parser failed to parse
            _ => None,
        }
    }
}
