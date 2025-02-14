use crate::handles::HandleCore;

impl HandleCore for u16 {
    fn into_index(self) -> usize {
        self as usize
    }

    fn from_index(index: usize) -> Self {
        index as Self // Possible type confusion
    }
}

impl HandleCore for u8 {
    fn into_index(self) -> usize {
        self as usize
    }

    fn from_index(index: usize) -> Self {
        index as Self // Possible type confusion
    }
}
