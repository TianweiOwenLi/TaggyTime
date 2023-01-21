

// only foreground tasks matter; bg tasks must be accompanied by fg task.
pub enum tasks<'a> {
    Homework(&'a str),   // includes anything in-class
    Projects(&'a str),   // extracurriculars with product
    Learning(&'a str),   // extracurriculars without product
    Exercise,   // active exercise
    Food,
    Sleep,
    Logistic,   // anything else
    Network,    // only formal ones ---- rest counts as breaks
    Breaks,
}

