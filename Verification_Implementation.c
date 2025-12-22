/* verify_nvrtc.c */
#include <dlfcn.h>
#include <stdio.h>

int main() {
    // Attempt to load the NVRTC shared object
    void *handle = dlopen("libnvrtc.so", RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "[-] Failed to load libnvrtc: %s\n", dlerror());
        return 1;
    }
    printf("[+] Successfully linked libnvrtc. Handle at: %p\n", handle);
    dlclose(handle);
    return 0;
}