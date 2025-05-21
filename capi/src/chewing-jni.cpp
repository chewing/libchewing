#include "chewing-cpp.h"
#include <cstdio>
#include <jni.h>
#include <stdarg.h>
#include <string>
#include <vector>
static std::vector<std::string> gCandidates;
static std::vector<std::string> gBuffers;
static void jni_candidate_callback(const char *candidate)
{
    gCandidates.emplace_back(candidate);
}
static void jni_buffer_callback(const char *buffer)
{
    gBuffers.emplace_back(buffer);
}

static JavaVM *gJvm = nullptr;
static jobject gLoggerObj = nullptr;
static jmethodID gLoggerMethod = nullptr;

jint JNI_OnLoad(JavaVM *vm, void * /*reserved*/)
{
    gJvm = vm;
    return JNI_VERSION_1_6;
}

// Shim that forwards logs to the Java Logger object
static void jni_logger_shim(void *data, int level, const char *message)
{
    if (!gJvm || !data)
        return;
    JNIEnv *env = nullptr;
    gJvm->AttachCurrentThread((void **)&env, nullptr);
    jstring jmsg = env->NewStringUTF(message);
    env->CallVoidMethod((jobject)data, gLoggerMethod, level, jmsg);
    env->DeleteLocalRef(jmsg);
}

// Variadic shim for chewing_set_logger: formats into a buffer then forwards to
// jni_logger_shim
static void chewing_jni_variadic_logger(void *data, int level, const char *fmt,
                                        ...)
{
    char buf[1024];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    jni_logger_shim(data, level, buf);
}

extern "C" {

/*
 * Class:     com_example_chewing_ChewingJNI
 * Method:    init
 * Signature: (Ljava/lang/String;)J
 */
JNIEXPORT jlong JNICALL
Java_com_example_chewing_ChewingJNI_init(JNIEnv *env, jclass, jstring jDataPath)
{
    const char *path = env->GetStringUTFChars(jDataPath, nullptr);
    static CallbacksContext cb_ctx;
    cb_ctx.data_path = const_cast<char *>(path);
    cb_ctx.candidate_callback = jni_candidate_callback;
    cb_ctx.buffer_callback = jni_buffer_callback;
    cb_ctx.bopomofo_callback = jni_buffer_callback;
    cb_ctx.commit_callback = jni_buffer_callback;
    cb_ctx.print_func = nullptr;
    cb_ctx.logger_func = jni_logger_shim;
    cb_ctx.logger_data = gLoggerObj;
    ChewingContext *ctx = nullptr;
    chewing_init(&ctx, &cb_ctx);
    env->ReleaseStringUTFChars(jDataPath, path);
    return reinterpret_cast<jlong>(ctx);
}

/*
 * Class:     com_example_chewing_ChewingJNI
 * Method:    terminate
 * Signature: (J)V
 */
JNIEXPORT void JNICALL
Java_com_example_chewing_ChewingJNI_terminate(JNIEnv *, jclass, jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    chewing_terminate(&ctx);
}

/*
 * Dispatchers for all the handle_* functions
 */
#define HANDLE_FN(name)                                                        \
    JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_handle##name(   \
        JNIEnv *, jclass, jlong ctxPtr)                                        \
    {                                                                          \
        auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);                \
        chewing_handle_##name(ctx);                                            \
    }

HANDLE_FN(Down)
HANDLE_FN(Up)
HANDLE_FN(PageUp)
HANDLE_FN(PageDown)
HANDLE_FN(Enter)
HANDLE_FN(Space)
#undef HANDLE_FN

/*
 * Default handler (with key)
 */
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_handleDefault(
    JNIEnv *, jclass, jlong ctxPtr, jint key)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    chewing_handle_Default(ctx, key);
}

/*
 * Fetch candidate list as a String[]
 */
JNIEXPORT jobjectArray JNICALL
Java_com_example_chewing_ChewingJNI_getCandidates(JNIEnv *env, jclass,
                                                  jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    gCandidates.clear();
    display_candidates(ctx);
    jclass strCls = env->FindClass("java/lang/String");
    jobjectArray arr = env->NewObjectArray(gCandidates.size(), strCls, nullptr);
    for (size_t i = 0; i < gCandidates.size(); ++i) {
        env->SetObjectArrayElement(arr, i,
                                   env->NewStringUTF(gCandidates[i].c_str()));
    }
    return arr;
}

/*
 * Fetch preedit/text/commit buffers
 */
JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getPreeditBuffer(
    JNIEnv *env, jclass, jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    gBuffers.clear();
    display_preedit_buffer(ctx);
    if (!gBuffers.empty()) {
        return env->NewStringUTF(gBuffers.front().c_str());
    }
    return nullptr;
}
JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getTextBuffer(
    JNIEnv *env, jclass, jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    gBuffers.clear();
    display_text_buffer(ctx);
    if (!gBuffers.empty()) {
        return env->NewStringUTF(gBuffers.front().c_str());
    }
    return nullptr;
}
JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getCommitBuffer(
    JNIEnv *env, jclass, jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    gBuffers.clear();
    display_commit_buffer(ctx);
    if (!gBuffers.empty()) {
        return env->NewStringUTF(gBuffers.front().c_str());
    }
    return nullptr;
}

/*
 * Status checkers
 */
JNIEXPORT jboolean JNICALL
Java_com_example_chewing_ChewingJNI_keystrokeCheckIgnore(JNIEnv *, jclass,
                                                         jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    return chewing_keystroke_CheckIgnore(ctx) != 0;
}

JNIEXPORT jboolean JNICALL
Java_com_example_chewing_ChewingJNI_keystrokeCheckAbsorb(JNIEnv *, jclass,
                                                         jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    return chewing_keystroke_CheckAbsorb(ctx) != 0;
}

JNIEXPORT jboolean JNICALL
Java_com_example_chewing_ChewingJNI_commitCheck(JNIEnv *, jclass, jlong ctxPtr)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    return chewing_commit_Check(ctx) != 0;
}

// Configuration APIs
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_setCandPerPage(
    JNIEnv *, jclass, jlong ctxPtr, jint page)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    chewing_set_candPerPage(ctx, page);
}

JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_setMaxChiSymbolLen(
    JNIEnv *, jclass, jlong ctxPtr, jint len)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    chewing_set_maxChiSymbolLen(ctx, len);
}

JNIEXPORT jint JNICALL Java_com_example_chewing_ChewingJNI_KBStr2Num(
    JNIEnv *env, jclass, jstring jName)
{
    const char *name = env->GetStringUTFChars(jName, nullptr);
    jint val = chewing_KBStr2Num(name);
    env->ReleaseStringUTFChars(jName, name);
    return val;
}

JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_setKBType(
    JNIEnv *, jclass, jlong ctxPtr, jint kb)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    chewing_set_KBType(ctx, kb);
}

/*
 * Java calls this to register (or clear) a Java-side Logger.
 * The Java Logger must implement void log(int level, String message).
 */
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_registerLogger(
    JNIEnv *env, jclass, jlong ctxPtr, jobject logger)
{
    auto *ctx = reinterpret_cast<ChewingContext *>(ctxPtr);
    // Release old reference
    if (gLoggerObj) {
        env->DeleteGlobalRef(gLoggerObj);
        gLoggerObj = nullptr;
        gLoggerMethod = nullptr;
    }
    if (logger) {
        // Keep global ref and lookup method
        gLoggerObj = env->NewGlobalRef(logger);
        jclass cls = env->GetObjectClass(logger);
        gLoggerMethod = env->GetMethodID(cls, "log", "(ILjava/lang/String;)V");
        // Register shim as the C logger
        chewing_set_logger(ctx, chewing_jni_variadic_logger, gLoggerObj);
    } else {
        // Clear logger in C
        chewing_set_logger(ctx, nullptr, nullptr);
    }
}

} // extern "C"