use crate::user::User;
use rand::Rng;
struct RandomPicker<'u, U: for<'hrtb> User<'hrtb>> {
    /// left child: idx * 2 + 1
    /// right child: idx * 2 + 2
    binary_tree: Vec<Option<BinaryTreeNode<'u, U>>>,
}
fn left(i: usize) -> usize {
    i * 2 + 1
}
fn right(i: usize) -> usize {
    i * 2 + 2
}
fn parent(i: usize) -> usize {
    (i - 1) / 2
}

#[derive(Debug)]
enum BinaryTreeNode<'u, U>
where
    U: for<'hrtb> User<'hrtb>,
{
    One(usize),
    Two(usize),
    Leaf(&'u mut U),
}

impl<'u, U: for<'hrtb> User<'hrtb>> RandomPicker<'u, U> {
    pub(crate) fn new(iter: impl IntoIterator<Item = &'u mut U>) -> Self {
        let iter = iter.into_iter();
        let mut binary_tree = Vec::with_capacity(iter.size_hint().0 * 2);
        binary_tree.extend(iter.map(BinaryTreeNode::Leaf).map(Some));
        {
            let num_users = binary_tree.len();
            binary_tree.extend(std::iter::repeat_n((), num_users).map(|_| None));
            let (left, right) = binary_tree.split_at_mut(num_users);
            left.swap_with_slice(right);
            fn init_at<U: for<'u> User<'u>>(i: usize, v: &mut [Option<BinaryTreeNode<U>>]) {
                if let None = &v[i] {
                    init_at(crate::random_picker::left(i), v);
                    init_at(crate::random_picker::right(i), v);
                    let left = match &v[crate::random_picker::left(i)] {
                        Some(BinaryTreeNode::Two(u)) => *u,
                        Some(BinaryTreeNode::Leaf(u)) => u.ticket_count(),
                        _ => unreachable!(),
                    };
                    let right = match &v[crate::random_picker::right(i)] {
                        Some(BinaryTreeNode::Two(u)) => *u,
                        Some(BinaryTreeNode::Leaf(u)) => u.ticket_count(),
                        _ => unreachable!(),
                    };
                    v[i] = Some(BinaryTreeNode::Two(left + right))
                }
            }
            (0..num_users).for_each(|i| init_at(i, &mut binary_tree));
        }
        Self { binary_tree }
    }

    /// Each node is either one user or a set of users,
    /// as determined by `BinaryTreeNode`,
    /// each with which tagged total number of tickets associated with all its descendants.
    ///
    /// Drawing a lucky user is done via traversing down the tree via sequence of binary questions,
    /// based on left/right tickets count,
    /// till a node that is exactly one user is found.
    ///
    /// It might be the case some user might not admit any more prizes,
    /// in which case backtrace to the nearest parent of which both left and right are not empty,
    /// and use that other child.
    ///
    /// TODO
    /// implement entropy pool, maybe as simple as caching random numbers.
    fn draw_one(&mut self, rng: &mut impl Rng) {}

    /// Some nodes are not binary tree anymore: they have only one child.
    /// Trim them s.t. we don't have to traverse down the tree next time.
    ///
    /// N.B.
    /// The ticket counters are assumed to be valid and thus _not_ modified,
    /// i.e. the input index should point to `Option::<BinaryTreeNode>::None`
    fn compactify(&mut self, mut i: usize) {
        while self.binary_tree[i].is_none() {
            match &mut self.binary_tree[parent(i)] {
                None | Some(BinaryTreeNode::Leaf(_)) => panic!("Data strucutre inconsistent"),
                one @ Some(BinaryTreeNode::One(_)) => {
                    // meaning the node was in fact a `BinaryTreeNode::Leaf`,
                    // i.e. we just removed a `&mut impl User` from the tree,
                    // thus this is where trim indeed happens
                    one.take();
                },
                two @ &mut Some(BinaryTreeNode::Two(x)) => {
                    two.replace(BinaryTreeNode::One(x));
                },
            }
            i = parent(i);
        }
    }
}
