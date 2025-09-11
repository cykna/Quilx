pub struct SizedVec<T, const N: usize>(Vec<T>);

impl<T, const N: usize> SizedVec<T, N> {
    ///Creates a new vector. if `prealloc` is set to true, it will pre allocate the amount of bytes required for this vector to reach it's max size without reallocations
    pub fn new(prealloc: bool) -> Self {
        Self(if prealloc {
            Vec::with_capacity(N)
        } else {
            Vec::new()
        })
    }
}
