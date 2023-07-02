use std::{cell::RefCell, rc::Rc};

use serde::Serializer;

use super::{wrapper::{self, StructAction}, Hooks};

#[derive(Debug, Clone)]
pub struct Context<H: Hooks> {
    inner: Rc<RefCell<ContextInner<H>>>,
}

impl<H: Hooks> wrapper::Hooks for Context<H> {
    fn path_push(&self, segment: wrapper::PathSegment) {
        todo!()
    }

    fn path_pop(&self) {
        todo!()
    }

    fn before_serialize(&self) -> wrapper::Action {
        todo!()
    }

    fn before_struct<S:Serializer>(&self) -> Vec<StructAction<S>> {
        todo!()
    }
}

impl<H: Hooks> Context<H> {
    pub(super) fn new(hooks: H) -> Self {
        Self {
            inner: Rc::new(RefCell::new(ContextInner {
                path: Default::default(),
                hooks,
            })),
        }
    }

    pub(super) fn start(&self) {
        self.inner.borrow().hooks.start();
    }

    pub(super) fn end(&self) {
        self.inner.borrow().hooks.end();
    }

    // pub(super) fn enter_path(&self, name: String) -> PathDropGuard<H> {
    //     todo!()
    //     // self.path.segments.push(name);
    //     // PathDropGuard(self)
    // }

    // fn exit_path(&self) {
    //     todo!()
    //     // self.path.segments.pop();
    // }
}

#[derive(Debug)]
struct ContextInner<H: Hooks> {
    path: Path,
    hooks: H,
}

// #[derive(Debug)]
// enum Segment {
//     Struct { name: String },
//     Map,
// }

#[derive(Debug, Default)]
struct Path {
    segments: Vec<String>,
}

struct PathDropGuard<'a, H: Hooks>(&'a Context<H>);

impl<'a, H: Hooks> Drop for PathDropGuard<'a, H> {
    fn drop(&mut self) {}
}
