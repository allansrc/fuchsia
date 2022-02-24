// Copyright 2021 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This file contains the data types representing structured data which filesystems must expose.

#ifndef SRC_LIB_STORAGE_VFS_CPP_INSPECT_INSPECT_DATA_H_
#define SRC_LIB_STORAGE_VFS_CPP_INSPECT_INSPECT_DATA_H_

#ifndef __Fuchsia__
#error "Fuchsia-only header"
#endif

#include <lib/inspect/cpp/inspect.h>
#include <lib/zx/status.h>

#include <cstdint>
#include <string>

#include "src/lib/storage/block_client/cpp/block_device.h"

namespace fs_inspect {

// fs.info properties
struct InfoData {
  uint64_t id;
  uint64_t type;
  std::string name;
  uint64_t version_major;
  uint64_t version_minor;
  uint64_t oldest_minor_version;
  uint64_t block_size;
  uint64_t max_filename_length;

  // Inspect Property Names

  static constexpr char kPropId[] = "id";
  static constexpr char kPropType[] = "type";
  static constexpr char kPropName[] = "name";
  static constexpr char kPropVersionMajor[] = "version_major";
  static constexpr char kPropVersionMinor[] = "version_minor";
  static constexpr char kPropOldestMinorVersion[] = "oldest_minor_version";
  static constexpr char kPropBlockSize[] = "block_size";
  static constexpr char kPropMaxFilenameLength[] = "max_filename_length";
};

// fs.usage properties
struct UsageData {
  uint64_t total_bytes;
  uint64_t used_bytes;
  uint64_t total_nodes;
  uint64_t used_nodes;

  // Inspect Property Names

  static constexpr char kPropTotalBytes[] = "total_bytes";
  static constexpr char kPropUsedBytes[] = "used_bytes";
  static constexpr char kPropTotalNodes[] = "total_nodes";
  static constexpr char kPropUsedNodes[] = "used_nodes";
};

// fs.volume properties
struct VolumeData {
  struct SizeInfo {
    // Current size of the volume that FVM has allocated for the filesystem.
    uint64_t size_bytes;
    // Size limit set on the volume, if any. If unset, value will be 0.
    uint64_t size_limit_bytes;
    // Amount of space the volume can be extended by. Based on the volume byte limit, if set,
    // otherwise the maximum amount of available slices.
    uint64_t available_space_bytes;
  } size_info;

  // Amount of times extending the volume failed when more space was required.
  uint64_t out_of_space_events;

  // Helper function to create a `SizeInfo` using the Volume protocol from a block device.
  static zx::status<SizeInfo> GetSizeInfoFromDevice(const block_client::BlockDevice& device);

  // Inspect Property Names

  static constexpr char kPropSizeBytes[] = "size_bytes";
  static constexpr char kPropSizeLimitBytes[] = "size_limit_bytes";
  static constexpr char kPropAvailableSpaceBytes[] = "available_space_bytes";
  static constexpr char kPropOutOfSpaceEvents[] = "out_of_space_events";
};

namespace detail {
// Attach the values from the given InfoData object as properties to the inspector's root node.
void Attach(inspect::Inspector& insp, const InfoData& info);

// Attach the values from the given UsageData object as properties to the inspector's root node.
void Attach(inspect::Inspector& insp, const UsageData& usage);

// Attach the values from the given VolumeData object as properties to the inspector's root node.
void Attach(inspect::Inspector& insp, const VolumeData& volume);
}  // namespace detail

}  // namespace fs_inspect

#endif  // SRC_LIB_STORAGE_VFS_CPP_INSPECT_INSPECT_DATA_H_