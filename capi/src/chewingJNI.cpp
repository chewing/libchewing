
#include "chewing-simplified.h"

#include <jni.h>
#include <cstring>
#include <cstdlib>
#include <vector>


static JavaVM*   g_vm    = nullptr;
static jobject   g_listener = nullptr;
static jmethodID g_midOnPreedit, g_midOnBuffer,
    g_midOnCommit, g_midOnCandidates;

static cs_context_t g_ctx;

// helper to copy Java string → C string
static char* strdup_from_jstring(JNIEnv* env, jstring js) {
    const char* u = env->GetStringUTFChars(js, nullptr);
    char* copy = strdup(u);
    env->ReleaseStringUTFChars(js, u);
    return copy;
}

// C‐callbacks that forward into Java:
static void jni_onPreedit(const char* txt) {
    JNIEnv* env = nullptr;
    g_vm->AttachCurrentThread(&env, nullptr);
    jstring js = env->NewStringUTF(txt ? txt : "");
    env->CallVoidMethod(g_listener, g_midOnPreedit, js);
    env->DeleteLocalRef(js);
}

static void jni_onBuffer(const char* txt) {
    JNIEnv* env = nullptr;
    g_vm->AttachCurrentThread(&env, nullptr);
    jstring js = env->NewStringUTF(txt ? txt : "");
    env->CallVoidMethod(g_listener, g_midOnBuffer, js);
    env->DeleteLocalRef(js);
}

static void jni_onCommit(const char* txt) {
    JNIEnv* env = nullptr;
    g_vm->AttachCurrentThread(&env, nullptr);
    jstring js = env->NewStringUTF(txt ? txt : "");
    env->CallVoidMethod(g_listener, g_midOnCommit, js);
    env->DeleteLocalRef(js);
}

static void jni_onCandidates(int pageSize, int numPages,
                             int candidateOnPage, int totalChoices,
                             const char** candidates) {
    JNIEnv* env = nullptr;
    g_vm->AttachCurrentThread(&env, nullptr);

    // build Java String[]
    jclass strCls = env->FindClass("java/lang/String");
    jobjectArray arr = env->NewObjectArray(totalChoices, strCls, nullptr);
    for (int i = 0; i < totalChoices; ++i) {
        env->SetObjectArrayElement(
            arr, i,
            env->NewStringUTF(candidates[i] ? candidates[i] : "")
        );
    }

    env->CallVoidMethod(
        g_listener,
        g_midOnCandidates,
        pageSize, numPages, candidateOnPage, totalChoices,
        arr
    );
    env->DeleteLocalRef(arr);
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_abaltatech_keyboard_chinese_ChineseConverter_initChewing(
    JNIEnv* env, jobject /*thiz*/,
    jstring dataPath, jint pp, jint ml, jobject listener) {

    // grab and stash JavaVM + listener
    env->GetJavaVM(&g_vm);
    g_listener = env->NewGlobalRef(listener);
    jclass cls = env->GetObjectClass(listener);

    // look up listener methods
    g_midOnPreedit    = env->GetMethodID(cls, "onPreedit",    "(Ljava/lang/String;)V");
    g_midOnBuffer     = env->GetMethodID(cls, "onBuffer",     "(Ljava/lang/String;)V");
    g_midOnCommit     = env->GetMethodID(cls, "onCommit",     "(Ljava/lang/String;)V");
    g_midOnCandidates = env->GetMethodID(
        cls,
        "onCandidates",
        "(IIII[Ljava/lang/String;)V"
    );

    // fill cs_context_t
    std::memset(&g_ctx, 0, sizeof(g_ctx));
    g_ctx.config.data_path         = strdup_from_jstring(env, dataPath);
    g_ctx.config.cand_per_page     = pp;
    g_ctx.config.max_chi_symbol_len = ml;

    // hook the C callbacks
    g_ctx.callbacks.bopomofo      = jni_onPreedit;
    g_ctx.callbacks.buffer        = jni_onBuffer;
    g_ctx.callbacks.commit        = jni_onCommit;
    g_ctx.callbacks.candidate_info = jni_onCandidates;

    bool ok = cs_init(&g_ctx);
    return ok ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT void JNICALL
Java_com_abaltatech_keyboard_chinese_ChineseConverter_processKey(
    JNIEnv*, jobject, jchar key) {
    cs_process_key(static_cast<char>(key));
}

extern "C" JNIEXPORT void JNICALL
Java_com_abaltatech_keyboard_chinese_ChineseConverter_selectCandidate(
    JNIEnv*, jobject, jint idx) {
    cs_select_candidate(idx);
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_abaltatech_keyboard_chinese_ChineseConverter_terminateChewing(
    JNIEnv* env, jobject) {
    bool ok = cs_terminate();
    free(g_ctx.config.data_path);
    env->DeleteGlobalRef(g_listener);
    g_listener = nullptr;
    return ok ? JNI_TRUE : JNI_FALSE;
}
