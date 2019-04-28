use super::Fitness;

macro_rules! impl_fitness_for_signed {
    ($($ty:ty),+) => {
        $(
            impl Fitness for $ty {
                #[inline]
                fn is_upper_limit(&self) -> bool {
                    *self == Self::max_value()
                }

                #[inline]
                fn is_lower_limit(&self) -> bool {
                    *self == Self::min_value()
                }
            }
        )+
    };
}

macro_rules! impl_fitness_for_unsigned {
    ($($ty:ty),+) => {
        $(
            impl Fitness for $ty {
                #[inline]
                fn is_upper_limit(&self) -> bool {
                    *self == Self::max_value()
                }

                #[inline]
                fn is_lower_limit(&self) -> bool {
                    false
                }
            }
        )+
    };
}

impl_fitness_for_signed!(i8, i16, i32, i64, i128, isize);
impl_fitness_for_unsigned!(u8, u16, u32, u64, u128, usize);

impl Fitness for bool {
    #[inline]
    fn is_upper_limit(&self) -> bool {
        *self
    }

    #[inline]
    fn is_lower_limit(&self) -> bool {
        false
    }
}
