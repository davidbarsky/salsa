use salsa::Setter;

#[salsa::input]
struct MyInput {
    field: String,
}

#[test]
fn mutation() {
    loom::model(|| {
        let mut db_t1 = salsa::DatabaseImpl::default();
        let mut db_t2 = db_t1.clone();

        // Thread 1:
        let t1 = loom::thread::spawn(move || {
            let input = MyInput::new(&db_t1, "Hello".to_string());

            // Overwrite field with an empty String
            // and store the old value in my_string
            let mut my_string = input.set_field(&mut db_t1).to(String::new());
            my_string.push_str(" World!");

            // Set the field back to out initial String,
            // expecting to get the empty one back
            assert_eq!(input.set_field(&mut db_t1).to(my_string), "");

            // Check if the stored String is the one we expected
            assert_eq!(input.field(&db_t1), "Hello World!");
        });

        // Thread 2:
        // let t2 = loom::thread::spawn(move || {
        //     let input = MyInput::new(&db_t2, "Hello".to_string());

        //     // Overwrite field with an empty String
        //     // and store the old value in my_string
        //     let mut my_string = input.set_field(&mut db_t2).to(String::new());
        //     my_string.push_str(" World!");

        //     // Set the field back to out initial String,
        //     // expecting to get the empty one back
        //     assert_eq!(input.set_field(&mut db_t2).to(my_string), "");

        //     // Check if the stored String is the one we expected
        //     assert_eq!(input.field(&db_t2), "Hello World!");
        // });

        t1.join().expect("thread 1 should not panic");
        // t2.join().expect("thread 2 should not panic");
    });
}
