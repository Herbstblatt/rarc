int global_var = 3;
static int static_var = 5;

int global_func(void) {
    return global_var;
}

static int static_func(void) {
    return static_var;
}

int main(void) {
    return global_func() + static_func();
}
