use itertools::Itertools;
use regex::Regex;

use crate::utils::TranslateMetadata;

/// RDC - Runtime dependency checker

/// Generate the code
pub fn generate(externals: Vec<String>) -> String {
    if externals.len() == 0 {
        return "".to_string();
    }
    
    let externals = externals.iter().dedup().map(|x| x.clone()).collect::<Vec<_>>();
    let mut code = String::new();
    code += "# This is the runtime dependency checker. Please do not remove these lines.\n";
    code += format!("AMBER_RDC_CD=(\'{}\')\n", externals.join("\' \'")).as_str();
    code += include_str!("rdc.sh");
    code
}

pub fn scan_append_externs(strings: Vec<String>, meta: &mut TranslateMetadata) -> () {
    // source: https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html
    let bash_deps = vec![ "alias", "bind", "builtin", "caller", "command", "declare", "echo", "enable", "help", "let", "local", "logout", "mapfile", "printf", "read", "readarray", "source", "type", "typeset", "ulimit", "unalias" ];
    
    let remove_prefix_var = Regex::new(r"^(\w+=\S+( \w+=\S+)*|) +").unwrap();
    let remove_all_non_cmd = Regex::new(r" .*$").unwrap();
    
    for x in strings.iter() {
        let parsed = remove_prefix_var.replace(x, "").to_string();
        let parsed = remove_all_non_cmd.replace(&parsed, "").to_string();
        if x.len() != 0 && bash_deps.iter().find(|y| x.to_string() == y.to_string()).is_none() {
            meta.externs.push(parsed);
            break;
        }
    }
}