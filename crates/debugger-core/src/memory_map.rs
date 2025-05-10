use nix::unistd::Pid;

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("The permission string needs to be of length 4")]
    PermissionLength,
    #[error("Could not parse field {0} with base {1}")]
    IntegerField(&'static str, u32),
    #[error("Expected field {0} to be present")]
    ExpectedField(&'static str),
}

#[derive(Debug, Clone)]
pub struct MemoryMapPermissions {
    read: bool,
    write: bool,
    execute: bool,
    private: bool,
}

impl MemoryMapPermissions {
    fn from_str(value: &str) -> Result<Self, ParseError> {
        if value.len() != 4 {
            return Err(ParseError::PermissionLength);
        }

        let chars: Vec<char> = value.chars().collect();

        let permissions = Self {
            read: chars[0] == 'r',
            write: chars[1] == 'w',
            execute: chars[2] == 'x',
            private: chars[3] == 'p',
        };

        Ok(permissions)
    }
}

#[derive(Debug, Clone)]
pub struct MemoryMap {
    pub range_from: u64,
    pub range_to: u64,
    pub permissions: MemoryMapPermissions,
    pub device_major: u64,
    pub device_minor: u64,
    pub offset: u64,
    pub inode_number: u64,
    pub pathname: Option<String>,
}

impl MemoryMap {
    fn parse_range(entry: &str, separator: char, radix: u32) -> Result<(u64, u64), ParseError> {
        let mut range = entry.split(separator);
        let range_from = range
            .next()
            .and_then(|s| u64::from_str_radix(s, radix).ok())
            .ok_or(ParseError::IntegerField("range_from", radix))?;
        let range_to = range
            .next()
            .and_then(|s| u64::from_str_radix(s, radix).ok())
            .ok_or(ParseError::IntegerField("range_to", radix))?;

        Ok((range_from, range_to))
    }

    fn from_str(entry: &str) -> Result<Self, ParseError> {
        let mut parts = entry.split(" ").filter(|s| !s.is_empty());
        let range = parts.next().ok_or(ParseError::ExpectedField("range"))?;
        let (range_from, range_to) = Self::parse_range(range, '-', 16)?;

        let permissions = parts
            .next()
            .ok_or(ParseError::ExpectedField("permissions"))?;
        let permissions = MemoryMapPermissions::from_str(permissions)?;

        let offset = parts
            .next()
            .and_then(|s| u64::from_str_radix(s, 16).ok())
            .ok_or(ParseError::ExpectedField("offset"))?;

        let device = parts.next().ok_or(ParseError::ExpectedField("device"))?;
        let (device_major, device_minor) = Self::parse_range(device, ':', 16)?;

        let inode_number = parts
            .next()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or(ParseError::ExpectedField("inode_number"))?;

        let pathname = parts.next().map(|s| s.to_owned());

        let memory_map = Self {
            range_from,
            range_to,
            permissions,
            device_major,
            device_minor,
            offset,
            inode_number,
            pathname,
        };

        Ok(memory_map)
    }
}

#[derive(Debug)]
pub struct ProcMemoryMaps {
    memory_maps: Vec<MemoryMap>,
}

impl ProcMemoryMaps {
    pub fn from_pid(pid: Pid) -> Result<Self, std::io::Error> {
        let maps_content = std::fs::read_to_string(format!("/proc/{pid}/maps"))?;

        let memory_maps: Vec<MemoryMap> = maps_content
            .lines()
            .map(|line| MemoryMap::from_str(line).expect("no parsing errors to occur"))
            .collect();

        Ok(Self { memory_maps })
    }

    pub fn get_text_section(&self) -> &MemoryMap {
        self.memory_maps
            .iter()
            .find(|map| map.permissions.execute)
            .expect("there to be atleast one text section per process")
    }
}
