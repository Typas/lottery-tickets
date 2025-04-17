use crate::user::User;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
struct RandomPicker<'btn, U: for<'u> User<'u>> {
    /// left child: idx * 2 + 1
    /// right child: idx * 2 + 2
    binary_tree: Vec<BinaryTreeNode<'btn, U>>,
}
fn left(i: usize) -> usize {
    i * 2 + 1
}
fn right(i: usize) -> usize {
    i * 2 + 2
}

enum BinaryTreeNode<'btn, U>
where
    U: for<'u> User<'u>,
{
    One,
    Two(usize),
    Leaf(Rc<RefCell<&'btn mut U>>),
}

impl<'btn, U: for<'u> User<'u>> RandomPicker<'btn, U> {
    pub(crate) fn new(iter: impl IntoIterator<Item = &'btn mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
        binary_tree.extend(iter.map(RefCell::new).map(Rc::new).map(BinaryTreeNode::Leaf));
        {
            let items = binary_tree.len();
            binary_tree.extend(std::iter::repeat_n((), items).map(|_| BinaryTreeNode::One));
            let (left, right) = binary_tree.split_at_mut(items);
            left.swap_with_slice(right);
            fn init_at<U: for<'u> User<'u>>(i: usize, v: &mut [BinaryTreeNode<U>]) {
                if let BinaryTreeNode::One = &v[i] {
                    init_at(crate::random_picker::left(i), v);
                    init_at(crate::random_picker::right(i), v);
                    let left = match &v[crate::random_picker::left(i)] {
                        BinaryTreeNode::Two(u) => *u,
                        BinaryTreeNode::Leaf(u) => u.borrow().ticket_count(),
                        _ => unreachable!(),
                    };
                    let right = match &v[crate::random_picker::right(i)] {
                        BinaryTreeNode::Two(u) => *u,
                        BinaryTreeNode::Leaf(u) => u.borrow().ticket_count(),
                        _ => unreachable!(),
                    };
                    v[i] = BinaryTreeNode::Two(left + right)
                }
            }
            (0..items).for_each(|i| init_at(i, &mut binary_tree));
        }
        Self { binary_tree }
    }
    fn draw_one(&mut self, rng: &mut impl Rng) {
    }
}
