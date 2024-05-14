#[derive(Clone, Copy)]
pub enum Owner {
    User,
    Group,
    Other,
}

impl Owner {
    pub fn masks(&self) -> [u32; 3] {
        match self {
            Owner::User => [0o400, 0o200, 0o100],
            Owner::Group => [0o40, 0o20, 0o10],
            Owner::Other => [0o4, 0o2, 0o1],
        }
    }
}
