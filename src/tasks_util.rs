
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tasks {            // SPECIAL is totally not an allusion to Fallout
    Sleep,                  // naps do count
    Projects(String),       // extracurriculars + learning
    Exercise,               // physical exercise
    Classwork(String),      // only the mandatory ones
    Interaction,            // interactions with humans
    Activity,               // fun!
    Logistic(String),               // anything else + meta
}

