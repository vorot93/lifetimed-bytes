use bytes::Buf;
use core::{
    borrow::Borrow,
    cmp,
    iter::FromIterator,
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, RangeBounds},
};

#[derive(Clone, Debug, Default, Hash)]
pub struct Bytes<'b> {
    inner: bytes::Bytes,
    _marker: PhantomData<&'b ()>,
}

impl<'b> Bytes<'b> {
    pub const fn new() -> Self {
        Self {
            inner: bytes::Bytes::new(),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        self.inner.slice(range).into()
    }

    pub fn slice_ref(&self, subset: &[u8]) -> Self {
        self.inner.slice_ref(subset).into()
    }

    #[must_use = "consider Bytes::truncate if you don't need the other half"]
    pub fn split_off(&mut self, at: usize) -> Self {
        self.inner.split_off(at).into()
    }

    #[must_use = "consider Bytes::advance if you don't need the other half"]
    pub fn split_to(&mut self, at: usize) -> Self {
        self.inner.split_to(at).into()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.inner.truncate(len)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    fn as_slice(&'b self) -> &'b [u8] {
        self.inner.borrow()
    }
}

impl<'b> Buf for Bytes<'b> {
    fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.as_slice()
    }

    fn advance(&mut self, cnt: usize) {
        self.inner.advance(cnt)
    }
}

impl<'b> Deref for Bytes<'b> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl<'b> AsRef<[u8]> for Bytes<'b> {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl<'b> Borrow<[u8]> for Bytes<'b> {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'b> From<&'b [u8]> for Bytes<'b> {
    fn from(raw: &'b [u8]) -> Self {
        // SAFETY: normally unsound, but we just move the lifetime from slice to struct itself
        let s = unsafe { transmute(raw) };
        Bytes {
            inner: bytes::Bytes::from_static(s),
            _marker: PhantomData,
        }
    }
}

impl<'b> From<&'b str> for Bytes<'b> {
    fn from(s: &'b str) -> Self {
        s.as_bytes().into()
    }
}

impl<'b> From<bytes::Bytes> for Bytes<'b> {
    fn from(inner: bytes::Bytes) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<'b> From<Vec<u8>> for Bytes<'b> {
    fn from(v: Vec<u8>) -> Self {
        bytes::Bytes::from(v).into()
    }
}

impl From<Bytes<'static>> for bytes::Bytes {
    fn from(l: Bytes<'static>) -> Self {
        l.inner
    }
}

impl<'b> FromIterator<u8> for Bytes<'b> {
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        bytes::Bytes::from_iter(into_iter).into()
    }
}

pub struct IntoIter<'b, T> {
    inner: bytes::buf::IntoIter<T>,
    _marker: PhantomData<&'b ()>,
}

impl<'b> Iterator for IntoIter<'b, bytes::Bytes> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'b> ExactSizeIterator for IntoIter<'b, bytes::Bytes> {}

impl<'b> IntoIterator for Bytes<'b> {
    type Item = u8;
    type IntoIter = IntoIter<'b, bytes::Bytes>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
            _marker: PhantomData,
        }
    }
}

macro_rules! forward_impls {
    ($t:ty) => {
        impl<'b> PartialEq<$t> for Bytes<'b> {
            fn eq(&self, other: &$t) -> bool {
                PartialEq::eq(&self.inner, other)
            }
        }

        impl<'b> PartialEq<Bytes<'b>> for $t {
            fn eq(&self, other: &Bytes<'b>) -> bool {
                PartialEq::eq(self, &other.inner)
            }
        }

        impl<'b> PartialOrd<$t> for Bytes<'b> {
            fn partial_cmp(&self, other: &$t) -> Option<cmp::Ordering> {
                PartialOrd::partial_cmp(&self.inner, other)
            }
        }

        impl<'b> PartialOrd<Bytes<'b>> for $t {
            fn partial_cmp(&self, other: &Bytes<'b>) -> Option<cmp::Ordering> {
                PartialOrd::partial_cmp(self, &other.inner)
            }
        }
    };
}

forward_impls!(bytes::Bytes);
forward_impls!([u8]);
forward_impls!(str);
forward_impls!(Vec<u8>);
forward_impls!(String);

impl<'a, 'b> PartialEq<Bytes<'a>> for Bytes<'b> {
    fn eq(&self, other: &Bytes<'a>) -> bool {
        PartialEq::eq(&self.inner, other)
    }
}

impl<'a, 'b> PartialOrd<Bytes<'a>> for Bytes<'b> {
    fn partial_cmp(&self, other: &Bytes<'a>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.inner, other)
    }
}

impl<'b> PartialEq<Bytes<'b>> for &[u8] {
    fn eq(&self, other: &Bytes<'b>) -> bool {
        PartialEq::eq(self, &other.inner)
    }
}

impl<'b> PartialOrd<Bytes<'b>> for &[u8] {
    fn partial_cmp(&self, other: &Bytes<'b>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self, &other.inner)
    }
}

impl<'b, const N: usize> PartialEq<Bytes<'b>> for [u8; N] {
    fn eq(&self, other: &Bytes<'b>) -> bool {
        PartialEq::eq(self as &[u8], &other.inner)
    }
}

impl<'b, const N: usize> PartialOrd<Bytes<'b>> for [u8; N] {
    fn partial_cmp(&self, other: &Bytes<'b>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self as &[u8], &other.inner)
    }
}

impl<'b> PartialEq<Bytes<'b>> for &str {
    fn eq(&self, other: &Bytes<'b>) -> bool {
        PartialEq::eq(self, &other.inner)
    }
}

impl<'b> PartialOrd<Bytes<'b>> for &str {
    fn partial_cmp(&self, other: &Bytes<'b>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(self, &other.inner)
    }
}

impl<'a, 'b, T: ?Sized> PartialEq<&'a T> for Bytes<'b>
where
    Bytes<'b>: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, 'b, T: ?Sized> PartialOrd<&'a T> for Bytes<'b>
where
    Bytes<'b>: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}

impl<'b> Eq for Bytes<'b> {}
impl<'b> Ord for Bytes<'b> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}
