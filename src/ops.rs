#[derive(Debug, Clone, Copy)]
pub enum Operator {
    // Standard operators
    ConditionalMove{ a: usize, b: usize, c: usize },
    ArrayIndex{ a: usize, b: usize, c: usize },
    ArrayAmendment{ a: usize, b: usize, c: usize },
    Addition{ a: usize, b: usize, c: usize },
    Multiplication{ a: usize, b: usize, c: usize },
    Division{ a: usize, b: usize, c: usize },
    NotAnd{ a: usize, b: usize, c: usize },
    // Other Operators
    Halt,
    Allocation{ b: usize, c: usize },
    Abandonment{ c: usize },
    Output{ c: usize },
    Input{ c: usize },
    LoadProgram{ b: usize, c: usize },
    // Special Operators
    Orthography{ a: usize, value: u32 }
}