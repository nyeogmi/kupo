#[derive(Clone, Copy, Debug)]
pub struct Located<T> {
    pub value: T,
    pub start: usize, 
    pub end: usize
}

impl<T> Located<T> {
    pub fn locmap<X>(self, f: impl Fn(T) -> X) -> Located<X> {
        Located {
            value: f(self.value), start: self.start, end: self.end
        }
    }

    pub fn replace<X>(&self, x: X) -> Located<X> {
        Located {
            value: x, start: self.start, end: self.end
        }
    }

    pub fn merge<X>(self, other: Located<X>) -> Located<(T, X)> {
        Located {
            value: (self.value, other.value),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn merge_r<X>(&self, other: Located<X>) -> Located<X> {
        self.location().merge(other).locmap(|(_, x)| x)
    }

    pub fn merge_l<X>(self, other: &Located<X>) -> Located<T> {
        self.merge(other.location()).locmap(|(t, _)| t)
    }

    pub fn location(&self) -> Located<()> {
        self.replace(())
    }
}