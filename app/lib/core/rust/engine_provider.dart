import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'engine_client.dart';

/// موتور فعال اپ — در تست‌ها با FakeEngineClient override می‌شود (سند ۰۸).
final engineProvider = Provider<EngineClient>((ref) => RustEngineClient());
