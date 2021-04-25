use super::Slot;
use crate::VarCnt;
use static_init::Lazy;
use std::{cell::Cell, cmp::max, marker::PhantomData};

pub(super) struct Node<CTX> {
    _ctx: PhantomData<CTX>,
    list: Box<[Slot]>,
    node: Lazy<Box<Node<CTX>>, Cell<Option<Box<dyn FnOnce() -> Box<Node<CTX>>>>>>,
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
            node: Lazy::new(Box::new(move || Box::new(Node::with_offset(offset + len)))),
            offset,
        }
    }

    fn node_or_init<'a>(&self) -> &Node<CTX> {
        &**Lazy::get(&self.node)
    }

    pub fn slot(&self, index: usize) -> Option<&Slot> {
        self.list
            .get(index - self.offset)
            .or_else(|| Lazy::try_get(&self.node).ok().and_then(|v| v.slot(index)))
    }

    pub fn slot_mut(&mut self, index: usize) -> Option<&mut Slot> {
        let node = &mut self.node;

        self.list
            .get_mut(index - self.offset)
            .or_else(move || Lazy::try_get_mut(node).ok().and_then(|v| v.slot_mut(index)))
    }

    pub fn slot_or_init(&self, index: usize) -> &Slot {
        self.list
            .get(index - self.offset)
            .unwrap_or_else(|| self.node_or_init().slot_or_init(index))
    }

    pub fn slot_or_init_mut(&mut self, index: usize) -> &mut Slot {
        let node = &mut self.node;

        self.list
            .get_mut(index - self.offset)
            .unwrap_or_else(move || Lazy::get_mut(node).slot_or_init_mut(index))
    }
}
