use pixll::dsl::shader_macros::compile_shader;

compile_shader!(TriangleShader, "shaders/triangle.vert.spv", "shaders/triangle.frag.spv");
