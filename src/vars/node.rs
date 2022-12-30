use super::Slot;
use crate::VarCnt;
use once_cell::sync::OnceCell;
use std::{cmp::max, marker::PhantomData};

pub(super) struct Node<CTX> {
    _ctx: PhantomData<CTX>,
    list: Box<[Slot]>,
    node: OnceCell<Box<Node<CTX>>>,
    offset: usize,
}

impl<CTX: VarCnt> Node<CTX> {
    pub fn with_offset(offset: usize) -> Self {
        let counter = CTX::var_cnt().cur();
        let len = max(counter.saturating_sub(offset), 256);

        let list = (0..len)
            .into_iter()
            .map(|_| Slot::default())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            _ctx: PhantomData,
            list,
            node: OnceCell::new(),
            offset,
        }
    }

    fn node_or_init(&self) -> &Node<CTX> {
        self.node
            .get_or_init(|| Box::new(Node::with_offset(self.offset + self.list.len())))
    }

    pub fn slot(&self, index: usize) -> Option<&Slot> {
        self.list
            .get(index - self.offset)
            .or_else(|| self.node.get().and_then(|v| v.slot(index)))
    }

    pub fn slot_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.list
            .get_mut(index - self.offset)
            .or_else(|| self.node.get_mut().and_then(|v| v.slot_mut(index)))
    }

    pub fn slot_or_init(&self, index: usize) -> &Slot {
        self.list
            .get(index - self.offset)
            .unwrap_or_else(|| self.node_or_init().slot_or_init(index))
    }

    pub fn slot_or_init_mut(&mut self, index: usize) -> &mut Slot {
        let node = &mut self.node;
        let size = self.offset + self.list.len();

        self.list.get_mut(index - self.offset).unwrap_or_else(|| {
            if node.get_mut().is_none() && node.set(Box::new(Node::with_offset(size))).is_err() {
                unreachable!("node set");
            }

            node.get_mut().expect("node").slot_or_init_mut(index)
        })
    }
}
