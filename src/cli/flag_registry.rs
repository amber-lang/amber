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
    flags: HashMap<String, Flag>
}

impl FlagRegistry {
    #[inline]
    pub fn new() -> Self {
        FlagRegistry {
            flags: HashMap::new()
        }
    }

    #[inline]
    pub fn get_flag(&self, name: impl AsRef<str>) -> Option<&Flag> {
        self.flags.get(name.as_ref())
    }

    #[inline]
    pub fn flag_triggered(&self, name: impl AsRef<str>) -> bool {
        self.flags.get(name.as_ref()).is_some_and(|flag| flag.triggered)
    }

    #[inline]
    pub fn register(&mut self, name: impl AsRef<str>, takes_value: bool) {
        self.flags.insert(name.as_ref().to_string(), Flag::new(takes_value));
    }

    pub fn parse(&mut self, args: Vec<String>) -> Vec<String> {
        let mut result = vec![];
        let mut is_value_flag = None;
        for arg in args.iter() {
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
            result.push(arg.to_string())
        }
        result
    }
}