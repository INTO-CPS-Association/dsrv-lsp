

#[derive(Debug, Clone)]
pub struct Spanned<'a, T> {
    pub node: &'a T,
    pub start: usize,
    pub end: usize,
}

//Define wrapper for AST Nodes, that contrains a reference to the node and its start and end position in the source code
impl<'a, T> Spanned<'a, T> {
    pub fn new(node: &'a T, start: usize, end: usize) -> Self {
        Self { node, start, end }
    }
}



