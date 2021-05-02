use lifetimed_bytes::Bytes;

fn main() {
    let v = b"hello".to_vec();
    let b = Bytes::from(v.as_slice());

    drop(v);

    println!("oops: {:?}", b);
}
