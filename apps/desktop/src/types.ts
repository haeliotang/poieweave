// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

export interface HostInfo {
  version: string;
  dataDir: string;
  dbPath: string;
  sandboxRoot: string;
  platform: string;
  sourceUrl: string;
}
