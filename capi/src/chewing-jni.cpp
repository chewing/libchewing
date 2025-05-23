#include "chewing-cpp.h"
#include <cstdio>
#include <jni.h>
#include <stdarg.h>
#include <string>
#include <vector>
static std::vector<std::string> gCandidates;
static std::vector<std::string> gBuffers;
static void jni_candidate_callback(const int /*pageSize*/, int /*numPages*/, int /*candOnPage*/, int /*total*/, const char **candidates)
{
    gCandidates.clear();
    if (!candidates) return;
    for (int i = 0; candidates[i]; ++i) {
        gCandidates.emplace_back(candidates[i]);
    }
}
static void jni_buffer_callback(const char *buffer)
{
    if (buffer)
        gBuffers.emplace_back(buffer);
}
static void jni_bopomofo_callback(const char *buffer)
{
    if (buffer)
        gBuffers.emplace_back(buffer);
}
static void jni_commit_callback(const char *buffer)
{
    if (buffer)
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
static void jni_logger_shim(int level, const char *message)
{
    if (!gJvm || !gLoggerObj)
        return;
    JNIEnv *env = nullptr;
    gJvm->AttachCurrentThread((void **)&env, nullptr);
    jstring jmsg = env->NewStringUTF(message);
    env->CallVoidMethod(gLoggerObj, gLoggerMethod, level, jmsg);
    env->DeleteLocalRef(jmsg);
}

// Variadic shim for chewing_set_logger: formats into a buffer then forwards to
// jni_logger_shim
// No longer needed: variadic logger

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
    auto *appCtx = new ApplicationContext();
    appCtx->config_data.data_path        = strdup(path);
    appCtx->config_data.candPerPage      = 5;               // default or override via setCandPerPage()
    appCtx->config_data.maxChiSymbolLen  = 18;              // default or override via setMaxChiSymbolLen()
    appCtx->callbacks.candidate_info_callback = jni_candidate_callback;
    appCtx->callbacks.buffer_callback         = jni_buffer_callback;
    appCtx->callbacks.bopomofo_callback       = jni_bopomofo_callback;
    appCtx->callbacks.commit_callback         = jni_commit_callback;
    appCtx->callbacks.logger_func             = jni_logger_shim;

    // initialize libchewing
    chewing_init(appCtx);
    env->ReleaseStringUTFChars(jDataPath, path);
    return reinterpret_cast<jlong>(appCtx);
}

/*
 * Class:     com_example_chewing_ChewingJNI
 * Method:    terminate
 * Signature: (J)V
 */
JNIEXPORT void JNICALL
Java_com_example_chewing_ChewingJNI_terminate(JNIEnv *, jclass, jlong ctxPtr)
{
    // shut down chewing and free our ApplicationContext
    chewing_terminate();
    auto *appCtx = reinterpret_cast<ApplicationContext *>(ctxPtr);
    free(appCtx->config_data.data_path);
    delete appCtx;
}


// New key handlers using process_key and C++ wrapper API
#define DEF_HANDLE(name, code)                                 \
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_handle##name(\
    JNIEnv *, jclass, jlong) {                                \
    process_key(code);                                        \
}

DEF_HANDLE(Down,    '/')
DEF_HANDLE(Up,      '\\')
DEF_HANDLE(PageUp,  '[')
DEF_HANDLE(PageDown,']')
DEF_HANDLE(Enter,   '\n')  // or CHEWING_KEY_Enter if defined
DEF_HANDLE(Space,   ' ')
#undef DEF_HANDLE

/*
 * Default handler (with key)
 */
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_handleDefault(
    JNIEnv *, jclass, jlong /*ctxPtr*/, jint key)
{
    process_key(static_cast<char>(key));
}

/*
 * Select a candidate by index (JNI shim)
 */
JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_selectCandidate(
    JNIEnv *, jclass, jlong /*ctxPtr*/, jint index)
{
    select_candidate(static_cast<int>(index));
}


/*
 * Fetch candidate list as a String[]
 */
JNIEXPORT jobjectArray JNICALL
Java_com_example_chewing_ChewingJNI_getCandidates(JNIEnv *env, jclass, jlong /*ctxPtr*/)
{
    jclass strCls = env->FindClass("java/lang/String");
    jobjectArray arr = env->NewObjectArray(gCandidates.size(), strCls, nullptr);
    for (size_t i = 0; i < gCandidates.size(); ++i) {
        env->SetObjectArrayElement(arr, i,
                                   env->NewStringUTF(gCandidates[i].c_str()));
    }
    return arr;
}

JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getPreeditBuffer(
    JNIEnv *env, jclass, jlong /*ctxPtr*/)
{
    if (gBuffers.empty()) return nullptr;
    jstring s = env->NewStringUTF(gBuffers.front().c_str());
    gBuffers.clear();
    return s;
}
JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getTextBuffer(
    JNIEnv *env, jclass, jlong /*ctxPtr*/)
{
    if (gBuffers.empty()) return nullptr;
    jstring s = env->NewStringUTF(gBuffers.front().c_str());
    gBuffers.clear();
    return s;
}
JNIEXPORT jstring JNICALL Java_com_example_chewing_ChewingJNI_getCommitBuffer(
    JNIEnv *env, jclass, jlong /*ctxPtr*/)
{
    if (gBuffers.empty()) return nullptr;
    jstring s = env->NewStringUTF(gBuffers.front().c_str());
    gBuffers.clear();
    return s;
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
    auto *appCtx = reinterpret_cast<ApplicationContext *>(ctxPtr);
    appCtx->config_data.candPerPage = page;
}

JNIEXPORT void JNICALL Java_com_example_chewing_ChewingJNI_setMaxChiSymbolLen(
    JNIEnv *, jclass, jlong ctxPtr, jint len)
{
    auto *appCtx = reinterpret_cast<ApplicationContext *>(ctxPtr);
    appCtx->config_data.maxChiSymbolLen = len;
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
    auto *appCtx = reinterpret_cast<ApplicationContext *>(ctxPtr);
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
        appCtx->callbacks.logger_func = jni_logger_shim;
    } else {
        // Clear logger in C
        appCtx->callbacks.logger_func = nullptr;
    }
}

} // extern "C"