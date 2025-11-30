//! Vector handling utilities

use crate::{errors::Result, ZpRead, ZpWrite};

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
type VecIntoIter<T> = alloc::vec::IntoIter<T>;
#[cfg(feature = "std")]
type VecIntoIter<T> = std::vec::IntoIter<T>;

/// A generic vector container for ZeroProto
#[derive(Debug, Clone)]
pub struct Vector<T> {
    elements: Vec<T>,
}

impl<T> Vector<T> {
    /// Create a new empty vector
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    /// Create a vector with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self { elements: Vec::with_capacity(capacity) }
    }

    /// Get the number of elements in the vector
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> core::slice::Iter<T> {
        self.elements.iter()
    }

    /// Get a mutable iterator over the elements
    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        self.elements.iter_mut()
    }

    /// Get an element at the given index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.elements.get(index)
    }

    /// Get a mutable element at the given index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.elements.get_mut(index)
    }

    /// Push an element to the vector
    pub fn push(&mut self, element: T) {
        self.elements.push(element);
    }

    /// Remove the last element
    pub fn pop(&mut self) -> Option<T> {
        self.elements.pop()
    }

    /// Clear the vector
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Convert to a Vec<T>
    pub fn into_vec(self) -> Vec<T> {
        self.elements
    }

    /// Get a slice of the elements
    pub fn as_slice(&self) -> &[T] {
        &self.elements
    }

    /// Get a mutable slice of the elements
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.elements
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Vec<T>> for Vector<T> {
    fn from(elements: Vec<T>) -> Self {
        Self { elements }
    }
}

impl<T> From<Vector<T>> for Vec<T> {
    fn from(vector: Vector<T>) -> Self {
        vector.elements
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = VecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter_mut()
    }
}

impl<T> core::ops::Deref for Vector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<T> core::ops::DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl<T> Extend<T> for Vector<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.elements.extend(iter);
    }
}

impl<T> FromIterator<T> for Vector<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self { elements: iter.into_iter().collect() }
    }
}

/// Utilities for working with vectors in ZeroProto buffers
pub struct VectorUtils;

impl VectorUtils {
    /// Calculate the total size needed to serialize a vector
    pub fn calculate_serialized_size<T: ZpWrite>(elements: &[T]) -> usize {
        if elements.is_empty() {
            4 // Just the count field
        } else {
            4 + elements.len() * elements[0].size()
        }
    }

    /// Write a vector to a buffer
    pub fn write_vector<T: ZpWrite>(
        elements: &[T],
        buffer: &mut [u8],
        offset: usize,
    ) -> Result<usize> {
        let count = elements.len();
        
        // Write count
        crate::primitives::Endian::Little.write_u32(count as u32, buffer, offset);
        let mut current_offset = offset + 4;
        
        // Write elements
        for element in elements {
            element.write(buffer, current_offset)?;
            current_offset += element.size();
        }
        
        Ok(current_offset - offset)
    }

    /// Read a vector from a buffer
    pub fn read_vector<'a, T: ZpRead<'a>>(
        buffer: &'a [u8],
        offset: usize,
    ) -> Result<(Vec<T>, usize)> {
        // Read count
        let count = crate::primitives::Endian::Little.read_u32(buffer, offset) as usize;
        let mut current_offset = offset + 4;
        
        // Read elements
        let mut elements = Vec::with_capacity(count);
        for _ in 0..count {
            let element = T::read(buffer, current_offset)?;
            elements.push(element);
            current_offset += T::size();
        }
        
        Ok((elements, current_offset - offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::MessageBuilder;
    use crate::reader::MessageReader;
    
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn test_vector_basic_operations() {
        let mut vec = Vector::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(1), Some(&2));
    }

    #[test]
    fn test_vector_iteration() {
        let vec = Vector::from(vec![1, 2, 3, 4, 5]);
        
        let collected: Vec<_> = vec.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_vector_serialization_size() {
        let elements = vec![1u32, 2u32, 3u32];
        let size = VectorUtils::calculate_serialized_size(&elements);
        assert_eq!(size, 4 + 3 * 4); // count + 3 u32s
    }

    #[test]
    fn test_empty_vector_serialization() {
        let elements: Vec<u32> = vec![];
        let size = VectorUtils::calculate_serialized_size(&elements);
        assert_eq!(size, 4); // Just the count field
    }
}
