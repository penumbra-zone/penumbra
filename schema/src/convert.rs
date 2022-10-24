pub trait Input<In, T, Err> {
    fn input(input: In) -> Result<T, Err>;
}

pub trait Output<T, Out> {
    fn output(output: T) -> Out;
}

pub struct Builder<In, Out, Err> {
    #[allow(clippy::type_complexity)]
    registry: Vec<(TypeId, fn(In) -> Result<Out, Err>)>,
}

impl<In, Out, Err> Builder<In, Out, Err> {
    pub fn new() -> Self {
        Self {
            registry: Vec::new(),
        }
    }

    pub fn register<I: Input<In, T, Err>, O: Output<T, Out>, T: 'static>(&mut self) {
        const fn converter<I: Input<In, T, Err>, O: Output<T, Out>, In, Out, Err, T>(
        ) -> fn(In) -> Result<Out, Err> {
            fn convert<In, Out, Err, I, O, T>(input: In) -> Result<Out, Err>
            where
                I: Input<In, T, Err>,
                O: Output<T, Out>,
            {
                I::input(input).map(O::output)
            }

            convert::<In, Out, Err, I, O, T>
        }

        self.registry
            .push((TypeId::of::<T>(), converter::<I, O, In, Out, Err, T>()));
    }
}

impl<In, Out, Err> Default for Builder<In, Out, Err> {
    fn default() -> Self {
        Self::new()
    }
}

impl<In, Out, Err> From<Builder<In, Out, Err>> for Converter<In, Out, Err> {
    fn from(mut builder: Builder<In, Out, Err>) -> Self {
        builder.registry.sort_by_key(|(id, _)| *id);
        Self {
            registry: builder.registry,
        }
    }
}

pub struct Converter<In, Out, Err> {
    #[allow(clippy::type_complexity)]
    registry: Vec<(TypeId, fn(In) -> Result<Out, Err>)>,
}

impl<In, Out, Err> Converter<In, Out, Err> {
    pub fn convert<T: 'static>(&self, input: In) -> Option<Result<Out, Err>> {
        self.registry
            .binary_search_by_key(&TypeId::of::<T>(), |(id, _)| *id)
            .map(|index| (self.registry[index].1)(input))
            .ok()
    }
}

fn test() {
    struct BytesTruncated;
    struct Displayed;

    impl Input<&[u8], u64, Infallible> for BytesTruncated {
        fn input(input: &[u8]) -> Result<u64, Infallible> {
            let mut bytes = [0u8; 8];
            for (to, from) in bytes.as_mut_slice().iter_mut().zip(input.iter()) {
                *to = *from;
            }
            Ok(u64::from_le_bytes(bytes))
        }
    }

    impl<T: Display> Output<T, String> for Displayed {
        fn output(output: T) -> String {
            format!("{}", output)
        }
    }

    let converter = root::converter::<&[u8], BytesTruncated, Displayed, String, Infallible>();

    let output = converter.convert::<u64>(&[1]);
}
