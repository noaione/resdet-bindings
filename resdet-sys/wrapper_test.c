void f() {
    (void)__builtin_signbit(1.0);
}

int main() {
    f();
    return 0;
}
