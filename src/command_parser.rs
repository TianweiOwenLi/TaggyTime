


pub enum Command {
    Tag(String, Vec<Command>),
    Atomic(String),
}

impl Command {

    /// parses a string into a command structure
    pub fn parse(s: String) -> Option<Self> {
        let n = s.len();
        let mut s_chr = s.chars();

        let fst_idx: Option<usize> = s_chr.position(|c| c == '(');
        let lst_idx: Option<usize> = 
            if s_chr.last() == Some(')') {Some(n)} else {None};

        match (fst_idx, lst_idx) {
            (Some(i), Some(j)) => {
                let mut v = vec![];
                let internal: Vec<&str> = s[(i+1)..(j-1)].split(",").collect();
                
                for item in internal {
                    match Command::parse(item.to_string()) {
                        Some(s) => v.push(s),
                        _ => return None, // premature end if fail to parse
                    }
                }

                Some(Command::Tag(
                    s[..i].trim().to_string(), 
                    v,
                ))
            }, 
            (None, None) => Some(Command::Atomic(s.clone())),
            _ => None // parenthesis mismatch
        }

    }
}
