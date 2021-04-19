use super::{CellOpt, UntypedPtr};
use crate::VARIABLE_COUNTER;
use std::{cmp::max, sync::atomic::Ordering::Relaxed};

pub(super) struct Node {
    list: Box<[CellOpt<UntypedPtr>]>,
    offset: usize,
    node: CellOpt<Box<Node>>,
}

impl Node {
    pub fn with_offset(offset: usize) -> Self {
        let counter = VARIABLE_COUNTER.load(Relaxed);
        let len = max(counter.saturating_sub(offset), 256);

        let list = (0..len)
            .into_iter()
            .map(|_| CellOpt::new(None))
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            list,
            node: CellOpt::new(None),
            offset,
        }
    }

    fn node_or_init<'a>(&self) -> &Node {
        self.node.get().unwrap_or_else(|| {
            self.node
                .replace(Some(new_node(self.offset, self.list.len())));

            self.node.get().unwrap()
        })
    }

    fn node_or_init_mut(&mut self) -> &mut Node {
        let o = self.node.get_mut();

        if o.is_none() {
            *o = Some(new_node(self.offset, self.list.len()));
        }

        o.as_mut().unwrap()
    }

    #[inline]
    pub fn slot(&self, index: usize) -> Option<&CellOpt<UntypedPtr>> {
        self.list
            .get(index - self.offset)
            .or_else(|| self.node.get().and_then(|v| v.slot(index)))
    }

    pub fn slot_mut(&mut self, index: usize) -> Option<&mut CellOpt<UntypedPtr>> {
        let node = &mut self.node;

        self.list
            .get_mut(index - self.offset)
            .or_else(move || node.get_mut().as_mut().and_then(|v| v.slot_mut(index)))
    }

    pub fn slot_or_init<'a>(&self, index: usize) -> &CellOpt<UntypedPtr> {
        self.list
            .get(index - self.offset)
            .unwrap_or_else(|| self.node_or_init().slot_or_init(index))
    }

    #[allow(dead_code)]
    pub fn slot_or_init_mut(&mut self, index: usize) -> &mut CellOpt<UntypedPtr> {
        let local_index = index - self.offset;

        if local_index < self.list.len() {
            unsafe { self.list.get_unchecked_mut(local_index) }
        } else {
            self.node_or_init_mut().slot_or_init_mut(index)
        }
    }
}

#[inline]
fn new_node(offset: usize, list_len: usize) -> Box<Node> {
    Box::new(Node::with_offset(offset + list_len))
}
