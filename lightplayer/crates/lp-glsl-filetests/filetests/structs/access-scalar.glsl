// test run
// target riscv32.fixed32

// ============================================================================
// Scalar Member Access
// ============================================================================

struct Point {
    float x;
    float y;
};

float test_access_scalar_point_x() {
    Point p = Point(1.0, 2.0);
    return p.x; // Access x member
    // Should be 1.0
}

// run: test_access_scalar_point_x() ~= 1.0

float test_access_scalar_point_y() {
    Point p = Point(3.0, 4.0);
    return p.y; // Access y member
    // Should be 4.0
}

// run: test_access_scalar_point_y() ~= 4.0

struct Color {
    float r;
    float g;
    float b;
};

float test_access_scalar_color_r() {
    Color c = Color(0.1, 0.2, 0.3);
    return c.r; // Access r member
    // Should be 0.1
}

// run: test_access_scalar_color_r() ~= 0.1

float test_access_scalar_color_g() {
    Color c = Color(0.5, 0.7, 0.9);
    return c.g; // Access g member
    // Should be 0.7
}

// run: test_access_scalar_color_g() ~= 0.7

float test_access_scalar_color_b() {
    Color c = Color(1.0, 0.5, 0.0);
    return c.b; // Access b member
    // Should be 0.0
}

// run: test_access_scalar_color_b() ~= 0.0

struct Triangle {
    float a;
    float b;
    float c;
};

float test_access_scalar_triangle_a() {
    Triangle t = Triangle(3.0, 4.0, 5.0);
    return t.a; // Access a member
    // Should be 3.0
}

// run: test_access_scalar_triangle_a() ~= 3.0

float test_access_scalar_triangle_sum() {
    Triangle t = Triangle(1.0, 2.0, 3.0);
    return t.a + t.b + t.c; // Access all members
    // Should be 6.0
}

// run: test_access_scalar_triangle_sum() ~= 6.0

struct Person {
    int age;
    float height;
    bool isStudent;
};

int test_access_scalar_person_age() {
    Person p = Person(25, 175.5, true);
    return p.age; // Access age member
    // Should be 25
}

// run: test_access_scalar_person_age() == 25

float test_access_scalar_person_height() {
    Person p = Person(30, 180.0, false);
    return p.height; // Access height member
    // Should be 180.0
}

// run: test_access_scalar_person_height() ~= 180.0

bool test_access_scalar_person_is_student() {
    Person p = Person(20, 165.0, true);
    return p.isStudent; // Access isStudent member
    // Should be true
}

// run: test_access_scalar_person_is_student() == true

struct Circle {
    float radius;
};

float test_access_scalar_circle_radius() {
    Circle c = Circle(10.0);
    return c.radius; // Access radius member
    // Should be 10.0
}

// run: test_access_scalar_circle_radius() ~= 10.0

struct EmptyData {
    int id;
};

int test_access_scalar_empty_data_id() {
    EmptyData d = EmptyData(42);
    return d.id; // Access id member
    // Should be 42
}

// run: test_access_scalar_empty_data_id() == 42
