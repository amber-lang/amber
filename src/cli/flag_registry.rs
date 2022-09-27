use std::{collections::HashMap};

pub struct Flag {
    pub takes_value: bool,
    pub triggered: bool,
    pub value: Option<String>
}

impl Flag {
    pub fn new(takes_value: bool) -> Self {
        Flag {
            takes_value,
            triggered: false,
            value: None
        }
    }
}

pub struct FlagRegistry {
    flags: HashMap<String, Flag>,
    args: Vec<String>
}

impl Default for FlagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FlagRegistry {
    #[inline]
    pub fn new() -> Self {
        FlagRegistry {
            flags: HashMap::new(),
            args: vec![]
        }
    }

    #[inline]
    pub fn get_flag(&self, name: impl AsRef<str>) -> Option<&Flag> {
        self.flags.get(name.as_ref())
    }

    #[inline]
    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    #[inline]
    pub fn flag_triggered(&self, name: impl AsRef<str>) -> bool {
        match self.flags.get(name.as_ref()) {
            Some(flag) => flag.triggered,
            None => false
        }
    }

    #[inline]
    pub fn register(&mut self, name: impl AsRef<str>, takes_value: bool) {
        self.flags.insert(name.as_ref().to_string(), Flag::new(takes_value));
    }

    pub fn parse(&mut self, args: Vec<String>) -> Vec<String> {
        let mut result = vec![];
        let mut is_value_flag = None;
        let mut is_args = false;
        for arg in args.iter() {
            if is_args {
                self.args.push(arg.clone());
                continue;
            }
            if let Some(name) = &is_value_flag {
                let flag = self.flags.get_mut(name).unwrap();
                flag.value = Some(arg.clone());
                continue
            }
            if let Some(flag) = self.flags.get_mut(arg) {
                flag.triggered = true;
                if flag.takes_value {
                    is_value_flag = Some(arg.clone());
                }
                continue
            }
            if arg == "--" {
                is_args = true;
                continue
            }
            result.push(arg.to_string())
        }
        result
    }
}