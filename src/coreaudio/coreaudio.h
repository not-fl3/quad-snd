// copy-pasted from sokol-app.h
// world without sokol_* would be a dark place full of despair

#include <stdbool.h>
#include <stdint.h>

typedef uint32_t _saudio_AudioFormatID;
typedef uint32_t _saudio_AudioFormatFlags;
typedef int32_t _saudio_OSStatus;
typedef uint32_t _saudio_SMPTETimeType;
typedef uint32_t _saudio_SMPTETimeFlags;
typedef uint32_t _saudio_AudioTimeStampFlags;
typedef void* _saudio_CFRunLoopRef;
typedef void* _saudio_CFStringRef;
typedef void* _saudio_AudioQueueRef;

#define _saudio_kAudioFormatLinearPCM ('lpcm')
#define _saudio_kLinearPCMFormatFlagIsFloat (1U << 0)
#define _saudio_kAudioFormatFlagIsPacked (1U << 3)

typedef struct _saudio_AudioStreamBasicDescription {
    double mSampleRate;
    _saudio_AudioFormatID mFormatID;
    _saudio_AudioFormatFlags mFormatFlags;
    uint32_t mBytesPerPacket;
    uint32_t mFramesPerPacket;
    uint32_t mBytesPerFrame;
    uint32_t mChannelsPerFrame;
    uint32_t mBitsPerChannel;
    uint32_t mReserved;
} _saudio_AudioStreamBasicDescription;

typedef struct _saudio_AudioStreamPacketDescription {
    int64_t mStartOffset;
    uint32_t mVariableFramesInPacket;
    uint32_t mDataByteSize;
} _saudio_AudioStreamPacketDescription;

typedef struct _saudio_SMPTETime {
    int16_t mSubframes;
    int16_t mSubframeDivisor;
    uint32_t mCounter;
    _saudio_SMPTETimeType mType;
    _saudio_SMPTETimeFlags mFlags;
    int16_t mHours;
    int16_t mMinutes;
    int16_t mSeconds;
    int16_t mFrames;
} _saudio_SMPTETime;

typedef struct _saudio_AudioTimeStamp {
    double mSampleTime;
    uint64_t mHostTime;
    double mRateScalar;
    uint64_t mWordClockTime;
    _saudio_SMPTETime mSMPTETime;
    _saudio_AudioTimeStampFlags mFlags;
    uint32_t mReserved;
} _saudio_AudioTimeStamp;

typedef struct _saudio_AudioQueueBuffer {
    const uint32_t mAudioDataBytesCapacity;
    void* const mAudioData;
    uint32_t mAudioDataByteSize;
    void * mUserData;
    const uint32_t mPacketDescriptionCapacity;
    _saudio_AudioStreamPacketDescription* const mPacketDescriptions;
    uint32_t mPacketDescriptionCount;
} _saudio_AudioQueueBuffer;
typedef _saudio_AudioQueueBuffer* _saudio_AudioQueueBufferRef;

typedef void (*_saudio_AudioQueueOutputCallback)(void* user_data, _saudio_AudioQueueRef inAQ, _saudio_AudioQueueBufferRef inBuffer);

extern _saudio_OSStatus AudioQueueNewOutput(const _saudio_AudioStreamBasicDescription* inFormat, _saudio_AudioQueueOutputCallback inCallbackProc, void* inUserData, _saudio_CFRunLoopRef inCallbackRunLoop, _saudio_CFStringRef inCallbackRunLoopMode, uint32_t inFlags, _saudio_AudioQueueRef* outAQ);
extern _saudio_OSStatus AudioQueueDispose(_saudio_AudioQueueRef inAQ, bool inImmediate);
extern _saudio_OSStatus AudioQueueAllocateBuffer(_saudio_AudioQueueRef inAQ, uint32_t inBufferByteSize, _saudio_AudioQueueBufferRef* outBuffer);
extern _saudio_OSStatus AudioQueueEnqueueBuffer(_saudio_AudioQueueRef inAQ, _saudio_AudioQueueBufferRef inBuffer, uint32_t inNumPacketDescs, const _saudio_AudioStreamPacketDescription* inPacketDescs);
extern _saudio_OSStatus AudioQueueStart(_saudio_AudioQueueRef inAQ, const _saudio_AudioTimeStamp * inStartTime);
extern _saudio_OSStatus AudioQueueStop(_saudio_AudioQueueRef inAQ, bool inImmediate);
