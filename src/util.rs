use std::cmp::Ordering;

pub struct Ordered<T, F>(pub T, pub F);

impl<T, F:  for<'a> Fn(&'a T, &'a T) -> Ordering> PartialEq<Self> for Ordered<T, F> {
    fn eq(&self, other: &Self) -> bool {
        match &self.1(&self.0, &other.0) {
            Ordering::Equal => true,
            _ => false,
        }
    }
}
impl<T, F> Eq for Ordered<T, F> where Ordered<T, F>: PartialEq {}

impl<T, F:  for<'a> Fn(&'a T, &'a T) -> Ordering> PartialOrd for Ordered<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.1(&self.0, &other.0))
    }
}
impl<T, F> std::cmp::Ord for Ordered<T, F>
where 
    F: for<'a> Fn(&'a T, &'a T) -> Ordering,
    Ordered<T, F>: PartialOrd
{    
    fn cmp(&self, other: &Self) -> Ordering {
        self.1(&self.0, &other.0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn order_int() {
        use super::*;
        let int_cmp = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(
            Ordered(5i32, int_cmp).cmp(&Ordered(2i32,int_cmp)),
            std::cmp::Ordering::Greater
        );
    }
}