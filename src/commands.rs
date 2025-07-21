use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CommandTrie {
    children: HashMap<char, CommandTrie>,
    is_command: bool,
    command_name: Option<String>,
}

impl CommandTrie {
    pub fn new() -> Self {
        CommandTrie {
            children: HashMap::new(),
            is_command: false,
            command_name: None,
        }
    }

    pub fn insert(&mut self, command: &str) {
        let mut node = self;
        for ch in command.to_lowercase().chars() {
            node = node.children.entry(ch).or_insert_with(CommandTrie::new);
        }
        node.is_command = true;
        node.command_name = Some(command.to_string());
    }

    pub fn search(&self, prefix: &str) -> (bool, bool, Option<String>) {
        let mut node = self;
        for ch in prefix.to_lowercase().chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return (false, false, None),
            }
        }
        (true, node.is_command, node.command_name.clone())
    }
}

pub fn initialize_command_trie() -> CommandTrie {
    let mut trie = CommandTrie::new();
    
    let math_commands = [
        "sin", "cos", "tan", "asin", "acos", "atan",
        "log", "ln", "exp", "sqrt", "pi", "inv", "pow"
    ];

    let stack_commands = ["enter", "swap", "clx", "clr", "chs"];
    let prog_commands = ["lbl", "gto", "xeq", "rtn", "sst", "bst", "prgm"];
    let special_commands = ["arc", "eex", "fix", "sci", "eng"];
    let storage_commands = ["sto", "rcl"];

    for &command in math_commands.iter()
        .chain(stack_commands.iter())
        .chain(prog_commands.iter())
        .chain(special_commands.iter())
        .chain(storage_commands.iter()) {
        trie.insert(command);
    }
    
    trie
}