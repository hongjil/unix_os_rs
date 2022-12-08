use core::cmp::{max, min};
pub trait StepByOne {
    fn step(&mut self);
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct SimpleRange<T>
where
    T: StepByOne + Eq + Ord + Copy,
{
    start: T,
    end: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Eq + Ord + Copy,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
    pub fn get_start(&self) -> T {
        self.start
    }
    pub fn get_end(&self) -> T {
        self.end
    }
    // Returns a range if there is an intersection between two sets.
    pub fn intersect(&self, another: Self) -> Option<Self> {
        if (another.start <= self.start && self.start < another.end)
            || (self.start <= another.start && another.start < self.end)
        {
            Some(Self::new(
                min(self.start, another.start),
                max(self.end, another.end),
            ))
        } else {
            None
        }
    }
}

impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Eq + Ord + Copy,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.start, self.end)
    }
}

pub struct SimpleRangeIterator<T>
where
    T: StepByOne + PartialEq + PartialOrd + Copy,
{
    cur: T,
    end: T,
}

impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + PartialEq + PartialOrd + Copy,
{
    fn new(start: T, end: T) -> Self {
        Self { cur: start, end }
    }
}

impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + PartialEq + PartialOrd + Copy,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.cur >= self.end {
            None
        } else {
            let ret = self.cur;
            self.cur.step();
            Some(ret)
        }
    }
}
