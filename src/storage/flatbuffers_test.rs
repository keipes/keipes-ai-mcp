#[cfg(test)]
mod tests {
    use flatbuffers::FlatBufferBuilder;

    #[test]
    fn test_simple_flatbuffers_basics() {
        // Just test that FlatBuffers can create a simple buffer
        let mut builder = FlatBufferBuilder::new();
        
        // Create a simple string
        let hello_string = builder.create_string("Hello, FlatBuffers!");
        
        // This is just testing basic functionality - not building a complex structure
        println!("FlatBuffer string offset: {}", hello_string.value());
        
        // Get the finished data
        builder.finish_minimal(hello_string);
        let fb_bytes = builder.finished_data();
        
        println!("FlatBuffer serialized {} bytes", fb_bytes.len());
        
        // Just verify we got some reasonable data
        assert!(fb_bytes.len() > 10);
        assert!(fb_bytes.len() < 100); // Should be small for just a string
        
        println!("✅ FlatBuffers basic functionality test passed!");
    }

    #[test]
    fn test_flatbuffers_multiple_strings() {
        let mut builder = FlatBufferBuilder::new();
        
        // Create multiple strings
        let names = vec!["Alice", "Bob", "Charlie"];
        let mut offsets = Vec::new();
        
        for name in &names {
            offsets.push(builder.create_string(name));
        }
        
        println!("Created {} string offsets", offsets.len());
        
        // Finish with the last string for simplicity
        builder.finish_minimal(offsets[0]);
        let fb_bytes = builder.finished_data();
        
        println!("FlatBuffer with multiple strings: {} bytes", fb_bytes.len());
        
        assert!(fb_bytes.len() > 5);
        
        println!("✅ FlatBuffers multiple strings test passed!");
    }
}
