#[cfg(test)]
mod tests {
    use steit::{
        gen::{generators::CSharpGenerator, *},
        log::loggers::PrintLogger,
        steitize,
        types::List,
        Runtime, Serialize,
    };

    #[steitize(State)]
    #[derive(Debug)]
    struct Outer {
        #[steit(tag = 0)]
        foo: i32,
        #[steit(tag = 1)]
        bar: bool,
        #[steit(tag = 2)]
        inner: Inner,
    }

    #[steitize(State)]
    #[derive(Debug)]
    struct Inner {
        #[steit(tag = 0)]
        foo: i32,
        #[steit(tag = 1)]
        bar: bool,
    }

    #[steitize(State)]
    #[derive(Debug)]
    enum Multicase {
        #[steit(tag = 0)]
        FirstCase {
            #[steit(tag = 0)]
            foo: i32,
            #[steit(tag = 1)]
            bar: bool,
        },
        #[steit(tag = 1)]
        SecondCase {
            #[steit(tag = 0)]
            foo: i32,
            #[steit(tag = 1)]
            bar: bool,
        },
    }

    #[steitize(State)]
    struct Hello {
        #[steit(tag = 0)]
        numbers: List<i32>,
        #[steit(tag = 1, skip_state)]
        others: Vec<i32>,
    }

    #[steitize(State)]
    pub enum Action {
        #[steit(tag = 0)]
        Raw {
            #[steit(tag = 0)]
            log_entries: List<u8>,
        },
        #[steit(tag = 1)]
        Attack {
            #[steit(tag = 0)]
            attacker: u8,
            #[steit(tag = 1)]
            defender: u8,
            #[steit(tag = 2)]
            hits: List<Hit>,
        },
    }

    #[steitize(State)]
    pub struct Hit {
        #[steit(tag = 0)]
        before_attacking: Box<Action>,
        #[steit(tag = 1)]
        before_damaging: Box<Action>,
        #[steit(tag = 2)]
        damaging: Box<Action>,
        #[steit(tag = 3)]
        after_damaging: Box<Action>,
        #[steit(tag = 4)]
        after_attacking: Box<Action>,
        #[steit(tag = 5)]
        dummy: i32,
    }

    #[test]
    fn test() {
        let out_dir = env!("CSHARP_OUT_DIR");
        let generator = CSharpGenerator::new("Test1", out_dir);

        generator.generate::<Outer>().unwrap();
        generator.generate::<Multicase>().unwrap();
        generator.generate::<Hello>().unwrap();
        generator.generate::<Action>().unwrap();

        println!("\nHELLO!");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut hello = Hello::new(runtime);

        hello
            .set_numbers_with(|runtime| {
                let mut list = List::new(runtime);
                list.push(1);
                list.push(2);
                list.push(1337);
                list
            })
            .set_others(vec![-1, -2, 1337]);

        let mut bytes = Vec::new();
        hello.serialize(&mut bytes).unwrap();
        println!("serialized: {:?}", bytes);

        println!("\nOUTER");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut outer = Outer::new(runtime);

        outer.set_foo(127).set_bar(true).set_inner_with(|runtime| {
            let mut inner = Inner::new(runtime);
            inner.set_foo(22).set_bar(true);
            inner
        });

        outer.inner.set_foo(160);
        outer.set_inner_with(Inner::new);

        println!("\nENUM");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut multicase = Multicase::new(runtime);

        multicase.set_second_case_foo(68);

        println!("\nLIST #1");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut list = List::new(runtime);

        list.push_with(|runtime| {
            let mut inner = Inner::new(runtime);
            inner.set_foo(6);
            inner
        });

        list.push_with(|runtime| {
            let mut inner = Inner::new(runtime);
            inner.set_foo(77).set_bar(true);
            inner
        });

        list.push_with(Inner::new);
        list.get_mut(1).unwrap().set_foo(68);
        list.remove(0);

        println!("\nLIST #2");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut list = List::new(runtime);
        list.push(10i8);
        list.push(11);
        list.push(0);
        list.remove(1);

        println!("\nACTION!");

        let logger = PrintLogger::with_stdout();
        let runtime = Runtime::with_logger(Box::new(logger));

        let mut action = Action::new(runtime);

        action.set_attack_attacker(1);
        action.set_attack_defender(2);

        action.set_attack_hits_with(|runtime| {
            let mut hits = List::new(runtime);

            for dummy in 6..=9 {
                hits.push_with(|runtime| {
                    let mut hit = Hit::new(runtime);
                    hit.set_dummy(dummy);
                    hit
                })
            }

            hits
        });
    }
}
