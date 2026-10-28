// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$ConsoleEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String text) print,
    required TResult Function(McError err) error,
    required TResult Function(String code, String text) warning,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String text)? print,
    TResult? Function(McError err)? error,
    TResult? Function(String code, String text)? warning,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String text)? print,
    TResult Function(McError err)? error,
    TResult Function(String code, String text)? warning,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConsoleEvent_Print value) print,
    required TResult Function(ConsoleEvent_Error value) error,
    required TResult Function(ConsoleEvent_Warning value) warning,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConsoleEvent_Print value)? print,
    TResult? Function(ConsoleEvent_Error value)? error,
    TResult? Function(ConsoleEvent_Warning value)? warning,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConsoleEvent_Print value)? print,
    TResult Function(ConsoleEvent_Error value)? error,
    TResult Function(ConsoleEvent_Warning value)? warning,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ConsoleEventCopyWith<$Res> {
  factory $ConsoleEventCopyWith(
          ConsoleEvent value, $Res Function(ConsoleEvent) then) =
      _$ConsoleEventCopyWithImpl<$Res, ConsoleEvent>;
}

/// @nodoc
class _$ConsoleEventCopyWithImpl<$Res, $Val extends ConsoleEvent>
    implements $ConsoleEventCopyWith<$Res> {
  _$ConsoleEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ConsoleEvent_PrintImplCopyWith<$Res> {
  factory _$$ConsoleEvent_PrintImplCopyWith(_$ConsoleEvent_PrintImpl value,
          $Res Function(_$ConsoleEvent_PrintImpl) then) =
      __$$ConsoleEvent_PrintImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String text});
}

/// @nodoc
class __$$ConsoleEvent_PrintImplCopyWithImpl<$Res>
    extends _$ConsoleEventCopyWithImpl<$Res, _$ConsoleEvent_PrintImpl>
    implements _$$ConsoleEvent_PrintImplCopyWith<$Res> {
  __$$ConsoleEvent_PrintImplCopyWithImpl(_$ConsoleEvent_PrintImpl _value,
      $Res Function(_$ConsoleEvent_PrintImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? text = null,
  }) {
    return _then(_$ConsoleEvent_PrintImpl(
      text: null == text
          ? _value.text
          : text // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$ConsoleEvent_PrintImpl extends ConsoleEvent_Print {
  const _$ConsoleEvent_PrintImpl({required this.text}) : super._();

  @override
  final String text;

  @override
  String toString() {
    return 'ConsoleEvent.print(text: $text)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConsoleEvent_PrintImpl &&
            (identical(other.text, text) || other.text == text));
  }

  @override
  int get hashCode => Object.hash(runtimeType, text);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConsoleEvent_PrintImplCopyWith<_$ConsoleEvent_PrintImpl> get copyWith =>
      __$$ConsoleEvent_PrintImplCopyWithImpl<_$ConsoleEvent_PrintImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String text) print,
    required TResult Function(McError err) error,
    required TResult Function(String code, String text) warning,
  }) {
    return print(text);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String text)? print,
    TResult? Function(McError err)? error,
    TResult? Function(String code, String text)? warning,
  }) {
    return print?.call(text);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String text)? print,
    TResult Function(McError err)? error,
    TResult Function(String code, String text)? warning,
    required TResult orElse(),
  }) {
    if (print != null) {
      return print(text);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConsoleEvent_Print value) print,
    required TResult Function(ConsoleEvent_Error value) error,
    required TResult Function(ConsoleEvent_Warning value) warning,
  }) {
    return print(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConsoleEvent_Print value)? print,
    TResult? Function(ConsoleEvent_Error value)? error,
    TResult? Function(ConsoleEvent_Warning value)? warning,
  }) {
    return print?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConsoleEvent_Print value)? print,
    TResult Function(ConsoleEvent_Error value)? error,
    TResult Function(ConsoleEvent_Warning value)? warning,
    required TResult orElse(),
  }) {
    if (print != null) {
      return print(this);
    }
    return orElse();
  }
}

abstract class ConsoleEvent_Print extends ConsoleEvent {
  const factory ConsoleEvent_Print({required final String text}) =
      _$ConsoleEvent_PrintImpl;
  const ConsoleEvent_Print._() : super._();

  String get text;

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConsoleEvent_PrintImplCopyWith<_$ConsoleEvent_PrintImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ConsoleEvent_ErrorImplCopyWith<$Res> {
  factory _$$ConsoleEvent_ErrorImplCopyWith(_$ConsoleEvent_ErrorImpl value,
          $Res Function(_$ConsoleEvent_ErrorImpl) then) =
      __$$ConsoleEvent_ErrorImplCopyWithImpl<$Res>;
  @useResult
  $Res call({McError err});
}

/// @nodoc
class __$$ConsoleEvent_ErrorImplCopyWithImpl<$Res>
    extends _$ConsoleEventCopyWithImpl<$Res, _$ConsoleEvent_ErrorImpl>
    implements _$$ConsoleEvent_ErrorImplCopyWith<$Res> {
  __$$ConsoleEvent_ErrorImplCopyWithImpl(_$ConsoleEvent_ErrorImpl _value,
      $Res Function(_$ConsoleEvent_ErrorImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? err = null,
  }) {
    return _then(_$ConsoleEvent_ErrorImpl(
      err: null == err
          ? _value.err
          : err // ignore: cast_nullable_to_non_nullable
              as McError,
    ));
  }
}

/// @nodoc

class _$ConsoleEvent_ErrorImpl extends ConsoleEvent_Error {
  const _$ConsoleEvent_ErrorImpl({required this.err}) : super._();

  @override
  final McError err;

  @override
  String toString() {
    return 'ConsoleEvent.error(err: $err)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConsoleEvent_ErrorImpl &&
            (identical(other.err, err) || other.err == err));
  }

  @override
  int get hashCode => Object.hash(runtimeType, err);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConsoleEvent_ErrorImplCopyWith<_$ConsoleEvent_ErrorImpl> get copyWith =>
      __$$ConsoleEvent_ErrorImplCopyWithImpl<_$ConsoleEvent_ErrorImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String text) print,
    required TResult Function(McError err) error,
    required TResult Function(String code, String text) warning,
  }) {
    return error(err);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String text)? print,
    TResult? Function(McError err)? error,
    TResult? Function(String code, String text)? warning,
  }) {
    return error?.call(err);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String text)? print,
    TResult Function(McError err)? error,
    TResult Function(String code, String text)? warning,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(err);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConsoleEvent_Print value) print,
    required TResult Function(ConsoleEvent_Error value) error,
    required TResult Function(ConsoleEvent_Warning value) warning,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConsoleEvent_Print value)? print,
    TResult? Function(ConsoleEvent_Error value)? error,
    TResult? Function(ConsoleEvent_Warning value)? warning,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConsoleEvent_Print value)? print,
    TResult Function(ConsoleEvent_Error value)? error,
    TResult Function(ConsoleEvent_Warning value)? warning,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }
}

abstract class ConsoleEvent_Error extends ConsoleEvent {
  const factory ConsoleEvent_Error({required final McError err}) =
      _$ConsoleEvent_ErrorImpl;
  const ConsoleEvent_Error._() : super._();

  McError get err;

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConsoleEvent_ErrorImplCopyWith<_$ConsoleEvent_ErrorImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ConsoleEvent_WarningImplCopyWith<$Res> {
  factory _$$ConsoleEvent_WarningImplCopyWith(_$ConsoleEvent_WarningImpl value,
          $Res Function(_$ConsoleEvent_WarningImpl) then) =
      __$$ConsoleEvent_WarningImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String code, String text});
}

/// @nodoc
class __$$ConsoleEvent_WarningImplCopyWithImpl<$Res>
    extends _$ConsoleEventCopyWithImpl<$Res, _$ConsoleEvent_WarningImpl>
    implements _$$ConsoleEvent_WarningImplCopyWith<$Res> {
  __$$ConsoleEvent_WarningImplCopyWithImpl(_$ConsoleEvent_WarningImpl _value,
      $Res Function(_$ConsoleEvent_WarningImpl) _then)
      : super(_value, _then);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? text = null,
  }) {
    return _then(_$ConsoleEvent_WarningImpl(
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
      text: null == text
          ? _value.text
          : text // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$ConsoleEvent_WarningImpl extends ConsoleEvent_Warning {
  const _$ConsoleEvent_WarningImpl({required this.code, required this.text})
      : super._();

  @override
  final String code;
  @override
  final String text;

  @override
  String toString() {
    return 'ConsoleEvent.warning(code: $code, text: $text)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConsoleEvent_WarningImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.text, text) || other.text == text));
  }

  @override
  int get hashCode => Object.hash(runtimeType, code, text);

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConsoleEvent_WarningImplCopyWith<_$ConsoleEvent_WarningImpl>
      get copyWith =>
          __$$ConsoleEvent_WarningImplCopyWithImpl<_$ConsoleEvent_WarningImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(String text) print,
    required TResult Function(McError err) error,
    required TResult Function(String code, String text) warning,
  }) {
    return warning(code, text);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(String text)? print,
    TResult? Function(McError err)? error,
    TResult? Function(String code, String text)? warning,
  }) {
    return warning?.call(code, text);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(String text)? print,
    TResult Function(McError err)? error,
    TResult Function(String code, String text)? warning,
    required TResult orElse(),
  }) {
    if (warning != null) {
      return warning(code, text);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ConsoleEvent_Print value) print,
    required TResult Function(ConsoleEvent_Error value) error,
    required TResult Function(ConsoleEvent_Warning value) warning,
  }) {
    return warning(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ConsoleEvent_Print value)? print,
    TResult? Function(ConsoleEvent_Error value)? error,
    TResult? Function(ConsoleEvent_Warning value)? warning,
  }) {
    return warning?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ConsoleEvent_Print value)? print,
    TResult Function(ConsoleEvent_Error value)? error,
    TResult Function(ConsoleEvent_Warning value)? warning,
    required TResult orElse(),
  }) {
    if (warning != null) {
      return warning(this);
    }
    return orElse();
  }
}

abstract class ConsoleEvent_Warning extends ConsoleEvent {
  const factory ConsoleEvent_Warning(
      {required final String code,
      required final String text}) = _$ConsoleEvent_WarningImpl;
  const ConsoleEvent_Warning._() : super._();

  String get code;
  String get text;

  /// Create a copy of ConsoleEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConsoleEvent_WarningImplCopyWith<_$ConsoleEvent_WarningImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$PlotSpec {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $PlotSpecCopyWith<$Res> {
  factory $PlotSpecCopyWith(PlotSpec value, $Res Function(PlotSpec) then) =
      _$PlotSpecCopyWithImpl<$Res, PlotSpec>;
}

/// @nodoc
class _$PlotSpecCopyWithImpl<$Res, $Val extends PlotSpec>
    implements $PlotSpecCopyWith<$Res> {
  _$PlotSpecCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$PlotSpec_Line2DImplCopyWith<$Res> {
  factory _$$PlotSpec_Line2DImplCopyWith(_$PlotSpec_Line2DImpl value,
          $Res Function(_$PlotSpec_Line2DImpl) then) =
      __$$PlotSpec_Line2DImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<Series2D> series, Axes2D axes, Legend? legend});
}

/// @nodoc
class __$$PlotSpec_Line2DImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_Line2DImpl>
    implements _$$PlotSpec_Line2DImplCopyWith<$Res> {
  __$$PlotSpec_Line2DImplCopyWithImpl(
      _$PlotSpec_Line2DImpl _value, $Res Function(_$PlotSpec_Line2DImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? series = null,
    Object? axes = null,
    Object? legend = freezed,
  }) {
    return _then(_$PlotSpec_Line2DImpl(
      series: null == series
          ? _value._series
          : series // ignore: cast_nullable_to_non_nullable
              as List<Series2D>,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes2D,
      legend: freezed == legend
          ? _value.legend
          : legend // ignore: cast_nullable_to_non_nullable
              as Legend?,
    ));
  }
}

/// @nodoc

class _$PlotSpec_Line2DImpl extends PlotSpec_Line2D {
  const _$PlotSpec_Line2DImpl(
      {required final List<Series2D> series, required this.axes, this.legend})
      : _series = series,
        super._();

  final List<Series2D> _series;
  @override
  List<Series2D> get series {
    if (_series is EqualUnmodifiableListView) return _series;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_series);
  }

  @override
  final Axes2D axes;
  @override
  final Legend? legend;

  @override
  String toString() {
    return 'PlotSpec.line2D(series: $series, axes: $axes, legend: $legend)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_Line2DImpl &&
            const DeepCollectionEquality().equals(other._series, _series) &&
            (identical(other.axes, axes) || other.axes == axes) &&
            (identical(other.legend, legend) || other.legend == legend));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_series), axes, legend);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_Line2DImplCopyWith<_$PlotSpec_Line2DImpl> get copyWith =>
      __$$PlotSpec_Line2DImplCopyWithImpl<_$PlotSpec_Line2DImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return line2D(series, axes, legend);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return line2D?.call(series, axes, legend);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (line2D != null) {
      return line2D(series, axes, legend);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return line2D(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return line2D?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (line2D != null) {
      return line2D(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Line2D extends PlotSpec {
  const factory PlotSpec_Line2D(
      {required final List<Series2D> series,
      required final Axes2D axes,
      final Legend? legend}) = _$PlotSpec_Line2DImpl;
  const PlotSpec_Line2D._() : super._();

  List<Series2D> get series;
  Axes2D get axes;
  Legend? get legend;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_Line2DImplCopyWith<_$PlotSpec_Line2DImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_Scatter2DImplCopyWith<$Res> {
  factory _$$PlotSpec_Scatter2DImplCopyWith(_$PlotSpec_Scatter2DImpl value,
          $Res Function(_$PlotSpec_Scatter2DImpl) then) =
      __$$PlotSpec_Scatter2DImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<Series2D> series, Axes2D axes, Legend? legend});
}

/// @nodoc
class __$$PlotSpec_Scatter2DImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_Scatter2DImpl>
    implements _$$PlotSpec_Scatter2DImplCopyWith<$Res> {
  __$$PlotSpec_Scatter2DImplCopyWithImpl(_$PlotSpec_Scatter2DImpl _value,
      $Res Function(_$PlotSpec_Scatter2DImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? series = null,
    Object? axes = null,
    Object? legend = freezed,
  }) {
    return _then(_$PlotSpec_Scatter2DImpl(
      series: null == series
          ? _value._series
          : series // ignore: cast_nullable_to_non_nullable
              as List<Series2D>,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes2D,
      legend: freezed == legend
          ? _value.legend
          : legend // ignore: cast_nullable_to_non_nullable
              as Legend?,
    ));
  }
}

/// @nodoc

class _$PlotSpec_Scatter2DImpl extends PlotSpec_Scatter2D {
  const _$PlotSpec_Scatter2DImpl(
      {required final List<Series2D> series, required this.axes, this.legend})
      : _series = series,
        super._();

  final List<Series2D> _series;
  @override
  List<Series2D> get series {
    if (_series is EqualUnmodifiableListView) return _series;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_series);
  }

  @override
  final Axes2D axes;
  @override
  final Legend? legend;

  @override
  String toString() {
    return 'PlotSpec.scatter2D(series: $series, axes: $axes, legend: $legend)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_Scatter2DImpl &&
            const DeepCollectionEquality().equals(other._series, _series) &&
            (identical(other.axes, axes) || other.axes == axes) &&
            (identical(other.legend, legend) || other.legend == legend));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_series), axes, legend);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_Scatter2DImplCopyWith<_$PlotSpec_Scatter2DImpl> get copyWith =>
      __$$PlotSpec_Scatter2DImplCopyWithImpl<_$PlotSpec_Scatter2DImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return scatter2D(series, axes, legend);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return scatter2D?.call(series, axes, legend);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (scatter2D != null) {
      return scatter2D(series, axes, legend);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return scatter2D(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return scatter2D?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (scatter2D != null) {
      return scatter2D(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Scatter2D extends PlotSpec {
  const factory PlotSpec_Scatter2D(
      {required final List<Series2D> series,
      required final Axes2D axes,
      final Legend? legend}) = _$PlotSpec_Scatter2DImpl;
  const PlotSpec_Scatter2D._() : super._();

  List<Series2D> get series;
  Axes2D get axes;
  Legend? get legend;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_Scatter2DImplCopyWith<_$PlotSpec_Scatter2DImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_Bar2DImplCopyWith<$Res> {
  factory _$$PlotSpec_Bar2DImplCopyWith(_$PlotSpec_Bar2DImpl value,
          $Res Function(_$PlotSpec_Bar2DImpl) then) =
      __$$PlotSpec_Bar2DImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<String> categories, Float64List values, Axes2D axes});
}

/// @nodoc
class __$$PlotSpec_Bar2DImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_Bar2DImpl>
    implements _$$PlotSpec_Bar2DImplCopyWith<$Res> {
  __$$PlotSpec_Bar2DImplCopyWithImpl(
      _$PlotSpec_Bar2DImpl _value, $Res Function(_$PlotSpec_Bar2DImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? categories = null,
    Object? values = null,
    Object? axes = null,
  }) {
    return _then(_$PlotSpec_Bar2DImpl(
      categories: null == categories
          ? _value._categories
          : categories // ignore: cast_nullable_to_non_nullable
              as List<String>,
      values: null == values
          ? _value.values
          : values // ignore: cast_nullable_to_non_nullable
              as Float64List,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes2D,
    ));
  }
}

/// @nodoc

class _$PlotSpec_Bar2DImpl extends PlotSpec_Bar2D {
  const _$PlotSpec_Bar2DImpl(
      {required final List<String> categories,
      required this.values,
      required this.axes})
      : _categories = categories,
        super._();

  final List<String> _categories;
  @override
  List<String> get categories {
    if (_categories is EqualUnmodifiableListView) return _categories;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_categories);
  }

  @override
  final Float64List values;
  @override
  final Axes2D axes;

  @override
  String toString() {
    return 'PlotSpec.bar2D(categories: $categories, values: $values, axes: $axes)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_Bar2DImpl &&
            const DeepCollectionEquality()
                .equals(other._categories, _categories) &&
            const DeepCollectionEquality().equals(other.values, values) &&
            (identical(other.axes, axes) || other.axes == axes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(_categories),
      const DeepCollectionEquality().hash(values),
      axes);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_Bar2DImplCopyWith<_$PlotSpec_Bar2DImpl> get copyWith =>
      __$$PlotSpec_Bar2DImplCopyWithImpl<_$PlotSpec_Bar2DImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return bar2D(categories, values, axes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return bar2D?.call(categories, values, axes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (bar2D != null) {
      return bar2D(categories, values, axes);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return bar2D(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return bar2D?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (bar2D != null) {
      return bar2D(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Bar2D extends PlotSpec {
  const factory PlotSpec_Bar2D(
      {required final List<String> categories,
      required final Float64List values,
      required final Axes2D axes}) = _$PlotSpec_Bar2DImpl;
  const PlotSpec_Bar2D._() : super._();

  List<String> get categories;
  Float64List get values;
  Axes2D get axes;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_Bar2DImplCopyWith<_$PlotSpec_Bar2DImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_HistogramImplCopyWith<$Res> {
  factory _$$PlotSpec_HistogramImplCopyWith(_$PlotSpec_HistogramImpl value,
          $Res Function(_$PlotSpec_HistogramImpl) then) =
      __$$PlotSpec_HistogramImplCopyWithImpl<$Res>;
  @useResult
  $Res call({Float64List binEdges, Uint64List counts, Axes2D axes});
}

/// @nodoc
class __$$PlotSpec_HistogramImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_HistogramImpl>
    implements _$$PlotSpec_HistogramImplCopyWith<$Res> {
  __$$PlotSpec_HistogramImplCopyWithImpl(_$PlotSpec_HistogramImpl _value,
      $Res Function(_$PlotSpec_HistogramImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? binEdges = null,
    Object? counts = null,
    Object? axes = null,
  }) {
    return _then(_$PlotSpec_HistogramImpl(
      binEdges: null == binEdges
          ? _value.binEdges
          : binEdges // ignore: cast_nullable_to_non_nullable
              as Float64List,
      counts: null == counts
          ? _value.counts
          : counts // ignore: cast_nullable_to_non_nullable
              as Uint64List,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes2D,
    ));
  }
}

/// @nodoc

class _$PlotSpec_HistogramImpl extends PlotSpec_Histogram {
  const _$PlotSpec_HistogramImpl(
      {required this.binEdges, required this.counts, required this.axes})
      : super._();

  @override
  final Float64List binEdges;
  @override
  final Uint64List counts;
  @override
  final Axes2D axes;

  @override
  String toString() {
    return 'PlotSpec.histogram(binEdges: $binEdges, counts: $counts, axes: $axes)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_HistogramImpl &&
            const DeepCollectionEquality().equals(other.binEdges, binEdges) &&
            const DeepCollectionEquality().equals(other.counts, counts) &&
            (identical(other.axes, axes) || other.axes == axes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(binEdges),
      const DeepCollectionEquality().hash(counts),
      axes);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_HistogramImplCopyWith<_$PlotSpec_HistogramImpl> get copyWith =>
      __$$PlotSpec_HistogramImplCopyWithImpl<_$PlotSpec_HistogramImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return histogram(binEdges, counts, axes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return histogram?.call(binEdges, counts, axes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (histogram != null) {
      return histogram(binEdges, counts, axes);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return histogram(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return histogram?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (histogram != null) {
      return histogram(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Histogram extends PlotSpec {
  const factory PlotSpec_Histogram(
      {required final Float64List binEdges,
      required final Uint64List counts,
      required final Axes2D axes}) = _$PlotSpec_HistogramImpl;
  const PlotSpec_Histogram._() : super._();

  Float64List get binEdges;
  Uint64List get counts;
  Axes2D get axes;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_HistogramImplCopyWith<_$PlotSpec_HistogramImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_PolarImplCopyWith<$Res> {
  factory _$$PlotSpec_PolarImplCopyWith(_$PlotSpec_PolarImpl value,
          $Res Function(_$PlotSpec_PolarImpl) then) =
      __$$PlotSpec_PolarImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<Series2D> series});
}

/// @nodoc
class __$$PlotSpec_PolarImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_PolarImpl>
    implements _$$PlotSpec_PolarImplCopyWith<$Res> {
  __$$PlotSpec_PolarImplCopyWithImpl(
      _$PlotSpec_PolarImpl _value, $Res Function(_$PlotSpec_PolarImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? series = null,
  }) {
    return _then(_$PlotSpec_PolarImpl(
      series: null == series
          ? _value._series
          : series // ignore: cast_nullable_to_non_nullable
              as List<Series2D>,
    ));
  }
}

/// @nodoc

class _$PlotSpec_PolarImpl extends PlotSpec_Polar {
  const _$PlotSpec_PolarImpl({required final List<Series2D> series})
      : _series = series,
        super._();

  final List<Series2D> _series;
  @override
  List<Series2D> get series {
    if (_series is EqualUnmodifiableListView) return _series;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_series);
  }

  @override
  String toString() {
    return 'PlotSpec.polar(series: $series)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_PolarImpl &&
            const DeepCollectionEquality().equals(other._series, _series));
  }

  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_series));

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_PolarImplCopyWith<_$PlotSpec_PolarImpl> get copyWith =>
      __$$PlotSpec_PolarImplCopyWithImpl<_$PlotSpec_PolarImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return polar(series);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return polar?.call(series);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (polar != null) {
      return polar(series);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return polar(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return polar?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (polar != null) {
      return polar(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Polar extends PlotSpec {
  const factory PlotSpec_Polar({required final List<Series2D> series}) =
      _$PlotSpec_PolarImpl;
  const PlotSpec_Polar._() : super._();

  List<Series2D> get series;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_PolarImplCopyWith<_$PlotSpec_PolarImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_QuiverImplCopyWith<$Res> {
  factory _$$PlotSpec_QuiverImplCopyWith(_$PlotSpec_QuiverImpl value,
          $Res Function(_$PlotSpec_QuiverImpl) then) =
      __$$PlotSpec_QuiverImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {Float64List x,
      Float64List y,
      Float64List u,
      Float64List v,
      Axes2D axes});
}

/// @nodoc
class __$$PlotSpec_QuiverImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_QuiverImpl>
    implements _$$PlotSpec_QuiverImplCopyWith<$Res> {
  __$$PlotSpec_QuiverImplCopyWithImpl(
      _$PlotSpec_QuiverImpl _value, $Res Function(_$PlotSpec_QuiverImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? x = null,
    Object? y = null,
    Object? u = null,
    Object? v = null,
    Object? axes = null,
  }) {
    return _then(_$PlotSpec_QuiverImpl(
      x: null == x
          ? _value.x
          : x // ignore: cast_nullable_to_non_nullable
              as Float64List,
      y: null == y
          ? _value.y
          : y // ignore: cast_nullable_to_non_nullable
              as Float64List,
      u: null == u
          ? _value.u
          : u // ignore: cast_nullable_to_non_nullable
              as Float64List,
      v: null == v
          ? _value.v
          : v // ignore: cast_nullable_to_non_nullable
              as Float64List,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes2D,
    ));
  }
}

/// @nodoc

class _$PlotSpec_QuiverImpl extends PlotSpec_Quiver {
  const _$PlotSpec_QuiverImpl(
      {required this.x,
      required this.y,
      required this.u,
      required this.v,
      required this.axes})
      : super._();

  @override
  final Float64List x;
  @override
  final Float64List y;
  @override
  final Float64List u;
  @override
  final Float64List v;
  @override
  final Axes2D axes;

  @override
  String toString() {
    return 'PlotSpec.quiver(x: $x, y: $y, u: $u, v: $v, axes: $axes)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_QuiverImpl &&
            const DeepCollectionEquality().equals(other.x, x) &&
            const DeepCollectionEquality().equals(other.y, y) &&
            const DeepCollectionEquality().equals(other.u, u) &&
            const DeepCollectionEquality().equals(other.v, v) &&
            (identical(other.axes, axes) || other.axes == axes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(x),
      const DeepCollectionEquality().hash(y),
      const DeepCollectionEquality().hash(u),
      const DeepCollectionEquality().hash(v),
      axes);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_QuiverImplCopyWith<_$PlotSpec_QuiverImpl> get copyWith =>
      __$$PlotSpec_QuiverImplCopyWithImpl<_$PlotSpec_QuiverImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return quiver(x, y, u, v, axes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return quiver?.call(x, y, u, v, axes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (quiver != null) {
      return quiver(x, y, u, v, axes);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return quiver(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return quiver?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (quiver != null) {
      return quiver(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Quiver extends PlotSpec {
  const factory PlotSpec_Quiver(
      {required final Float64List x,
      required final Float64List y,
      required final Float64List u,
      required final Float64List v,
      required final Axes2D axes}) = _$PlotSpec_QuiverImpl;
  const PlotSpec_Quiver._() : super._();

  Float64List get x;
  Float64List get y;
  Float64List get u;
  Float64List get v;
  Axes2D get axes;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_QuiverImplCopyWith<_$PlotSpec_QuiverImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_SurfaceImplCopyWith<$Res> {
  factory _$$PlotSpec_SurfaceImplCopyWith(_$PlotSpec_SurfaceImpl value,
          $Res Function(_$PlotSpec_SurfaceImpl) then) =
      __$$PlotSpec_SurfaceImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {Float64List x,
      Float64List y,
      List<Float64List> z,
      Shading shading,
      Colormap colormap});
}

/// @nodoc
class __$$PlotSpec_SurfaceImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_SurfaceImpl>
    implements _$$PlotSpec_SurfaceImplCopyWith<$Res> {
  __$$PlotSpec_SurfaceImplCopyWithImpl(_$PlotSpec_SurfaceImpl _value,
      $Res Function(_$PlotSpec_SurfaceImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? x = null,
    Object? y = null,
    Object? z = null,
    Object? shading = null,
    Object? colormap = null,
  }) {
    return _then(_$PlotSpec_SurfaceImpl(
      x: null == x
          ? _value.x
          : x // ignore: cast_nullable_to_non_nullable
              as Float64List,
      y: null == y
          ? _value.y
          : y // ignore: cast_nullable_to_non_nullable
              as Float64List,
      z: null == z
          ? _value._z
          : z // ignore: cast_nullable_to_non_nullable
              as List<Float64List>,
      shading: null == shading
          ? _value.shading
          : shading // ignore: cast_nullable_to_non_nullable
              as Shading,
      colormap: null == colormap
          ? _value.colormap
          : colormap // ignore: cast_nullable_to_non_nullable
              as Colormap,
    ));
  }
}

/// @nodoc

class _$PlotSpec_SurfaceImpl extends PlotSpec_Surface {
  const _$PlotSpec_SurfaceImpl(
      {required this.x,
      required this.y,
      required final List<Float64List> z,
      required this.shading,
      required this.colormap})
      : _z = z,
        super._();

  @override
  final Float64List x;
  @override
  final Float64List y;
  final List<Float64List> _z;
  @override
  List<Float64List> get z {
    if (_z is EqualUnmodifiableListView) return _z;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_z);
  }

  @override
  final Shading shading;
  @override
  final Colormap colormap;

  @override
  String toString() {
    return 'PlotSpec.surface(x: $x, y: $y, z: $z, shading: $shading, colormap: $colormap)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_SurfaceImpl &&
            const DeepCollectionEquality().equals(other.x, x) &&
            const DeepCollectionEquality().equals(other.y, y) &&
            const DeepCollectionEquality().equals(other._z, _z) &&
            (identical(other.shading, shading) || other.shading == shading) &&
            (identical(other.colormap, colormap) ||
                other.colormap == colormap));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(x),
      const DeepCollectionEquality().hash(y),
      const DeepCollectionEquality().hash(_z),
      shading,
      colormap);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_SurfaceImplCopyWith<_$PlotSpec_SurfaceImpl> get copyWith =>
      __$$PlotSpec_SurfaceImplCopyWithImpl<_$PlotSpec_SurfaceImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return surface(x, y, z, shading, colormap);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return surface?.call(x, y, z, shading, colormap);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (surface != null) {
      return surface(x, y, z, shading, colormap);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return surface(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return surface?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (surface != null) {
      return surface(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Surface extends PlotSpec {
  const factory PlotSpec_Surface(
      {required final Float64List x,
      required final Float64List y,
      required final List<Float64List> z,
      required final Shading shading,
      required final Colormap colormap}) = _$PlotSpec_SurfaceImpl;
  const PlotSpec_Surface._() : super._();

  Float64List get x;
  Float64List get y;
  List<Float64List> get z;
  Shading get shading;
  Colormap get colormap;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_SurfaceImplCopyWith<_$PlotSpec_SurfaceImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_MeshImplCopyWith<$Res> {
  factory _$$PlotSpec_MeshImplCopyWith(
          _$PlotSpec_MeshImpl value, $Res Function(_$PlotSpec_MeshImpl) then) =
      __$$PlotSpec_MeshImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {Float64List x, Float64List y, List<Float64List> z, Colormap colormap});
}

/// @nodoc
class __$$PlotSpec_MeshImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_MeshImpl>
    implements _$$PlotSpec_MeshImplCopyWith<$Res> {
  __$$PlotSpec_MeshImplCopyWithImpl(
      _$PlotSpec_MeshImpl _value, $Res Function(_$PlotSpec_MeshImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? x = null,
    Object? y = null,
    Object? z = null,
    Object? colormap = null,
  }) {
    return _then(_$PlotSpec_MeshImpl(
      x: null == x
          ? _value.x
          : x // ignore: cast_nullable_to_non_nullable
              as Float64List,
      y: null == y
          ? _value.y
          : y // ignore: cast_nullable_to_non_nullable
              as Float64List,
      z: null == z
          ? _value._z
          : z // ignore: cast_nullable_to_non_nullable
              as List<Float64List>,
      colormap: null == colormap
          ? _value.colormap
          : colormap // ignore: cast_nullable_to_non_nullable
              as Colormap,
    ));
  }
}

/// @nodoc

class _$PlotSpec_MeshImpl extends PlotSpec_Mesh {
  const _$PlotSpec_MeshImpl(
      {required this.x,
      required this.y,
      required final List<Float64List> z,
      required this.colormap})
      : _z = z,
        super._();

  @override
  final Float64List x;
  @override
  final Float64List y;
  final List<Float64List> _z;
  @override
  List<Float64List> get z {
    if (_z is EqualUnmodifiableListView) return _z;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_z);
  }

  @override
  final Colormap colormap;

  @override
  String toString() {
    return 'PlotSpec.mesh(x: $x, y: $y, z: $z, colormap: $colormap)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_MeshImpl &&
            const DeepCollectionEquality().equals(other.x, x) &&
            const DeepCollectionEquality().equals(other.y, y) &&
            const DeepCollectionEquality().equals(other._z, _z) &&
            (identical(other.colormap, colormap) ||
                other.colormap == colormap));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(x),
      const DeepCollectionEquality().hash(y),
      const DeepCollectionEquality().hash(_z),
      colormap);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_MeshImplCopyWith<_$PlotSpec_MeshImpl> get copyWith =>
      __$$PlotSpec_MeshImplCopyWithImpl<_$PlotSpec_MeshImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return mesh(x, y, z, colormap);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return mesh?.call(x, y, z, colormap);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (mesh != null) {
      return mesh(x, y, z, colormap);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return mesh(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return mesh?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (mesh != null) {
      return mesh(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Mesh extends PlotSpec {
  const factory PlotSpec_Mesh(
      {required final Float64List x,
      required final Float64List y,
      required final List<Float64List> z,
      required final Colormap colormap}) = _$PlotSpec_MeshImpl;
  const PlotSpec_Mesh._() : super._();

  Float64List get x;
  Float64List get y;
  List<Float64List> get z;
  Colormap get colormap;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_MeshImplCopyWith<_$PlotSpec_MeshImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_ContourImplCopyWith<$Res> {
  factory _$$PlotSpec_ContourImplCopyWith(_$PlotSpec_ContourImpl value,
          $Res Function(_$PlotSpec_ContourImpl) then) =
      __$$PlotSpec_ContourImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {Float64List x, Float64List y, List<Float64List> z, Float64List levels});
}

/// @nodoc
class __$$PlotSpec_ContourImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_ContourImpl>
    implements _$$PlotSpec_ContourImplCopyWith<$Res> {
  __$$PlotSpec_ContourImplCopyWithImpl(_$PlotSpec_ContourImpl _value,
      $Res Function(_$PlotSpec_ContourImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? x = null,
    Object? y = null,
    Object? z = null,
    Object? levels = null,
  }) {
    return _then(_$PlotSpec_ContourImpl(
      x: null == x
          ? _value.x
          : x // ignore: cast_nullable_to_non_nullable
              as Float64List,
      y: null == y
          ? _value.y
          : y // ignore: cast_nullable_to_non_nullable
              as Float64List,
      z: null == z
          ? _value._z
          : z // ignore: cast_nullable_to_non_nullable
              as List<Float64List>,
      levels: null == levels
          ? _value.levels
          : levels // ignore: cast_nullable_to_non_nullable
              as Float64List,
    ));
  }
}

/// @nodoc

class _$PlotSpec_ContourImpl extends PlotSpec_Contour {
  const _$PlotSpec_ContourImpl(
      {required this.x,
      required this.y,
      required final List<Float64List> z,
      required this.levels})
      : _z = z,
        super._();

  @override
  final Float64List x;
  @override
  final Float64List y;
  final List<Float64List> _z;
  @override
  List<Float64List> get z {
    if (_z is EqualUnmodifiableListView) return _z;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_z);
  }

  @override
  final Float64List levels;

  @override
  String toString() {
    return 'PlotSpec.contour(x: $x, y: $y, z: $z, levels: $levels)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_ContourImpl &&
            const DeepCollectionEquality().equals(other.x, x) &&
            const DeepCollectionEquality().equals(other.y, y) &&
            const DeepCollectionEquality().equals(other._z, _z) &&
            const DeepCollectionEquality().equals(other.levels, levels));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(x),
      const DeepCollectionEquality().hash(y),
      const DeepCollectionEquality().hash(_z),
      const DeepCollectionEquality().hash(levels));

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_ContourImplCopyWith<_$PlotSpec_ContourImpl> get copyWith =>
      __$$PlotSpec_ContourImplCopyWithImpl<_$PlotSpec_ContourImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return contour(x, y, z, levels);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return contour?.call(x, y, z, levels);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (contour != null) {
      return contour(x, y, z, levels);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return contour(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return contour?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (contour != null) {
      return contour(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Contour extends PlotSpec {
  const factory PlotSpec_Contour(
      {required final Float64List x,
      required final Float64List y,
      required final List<Float64List> z,
      required final Float64List levels}) = _$PlotSpec_ContourImpl;
  const PlotSpec_Contour._() : super._();

  Float64List get x;
  Float64List get y;
  List<Float64List> get z;
  Float64List get levels;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_ContourImplCopyWith<_$PlotSpec_ContourImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_Plot3ImplCopyWith<$Res> {
  factory _$$PlotSpec_Plot3ImplCopyWith(_$PlotSpec_Plot3Impl value,
          $Res Function(_$PlotSpec_Plot3Impl) then) =
      __$$PlotSpec_Plot3ImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<Series3D> series, Axes3D axes});
}

/// @nodoc
class __$$PlotSpec_Plot3ImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_Plot3Impl>
    implements _$$PlotSpec_Plot3ImplCopyWith<$Res> {
  __$$PlotSpec_Plot3ImplCopyWithImpl(
      _$PlotSpec_Plot3Impl _value, $Res Function(_$PlotSpec_Plot3Impl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? series = null,
    Object? axes = null,
  }) {
    return _then(_$PlotSpec_Plot3Impl(
      series: null == series
          ? _value._series
          : series // ignore: cast_nullable_to_non_nullable
              as List<Series3D>,
      axes: null == axes
          ? _value.axes
          : axes // ignore: cast_nullable_to_non_nullable
              as Axes3D,
    ));
  }
}

/// @nodoc

class _$PlotSpec_Plot3Impl extends PlotSpec_Plot3 {
  const _$PlotSpec_Plot3Impl(
      {required final List<Series3D> series, required this.axes})
      : _series = series,
        super._();

  final List<Series3D> _series;
  @override
  List<Series3D> get series {
    if (_series is EqualUnmodifiableListView) return _series;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_series);
  }

  @override
  final Axes3D axes;

  @override
  String toString() {
    return 'PlotSpec.plot3(series: $series, axes: $axes)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_Plot3Impl &&
            const DeepCollectionEquality().equals(other._series, _series) &&
            (identical(other.axes, axes) || other.axes == axes));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, const DeepCollectionEquality().hash(_series), axes);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_Plot3ImplCopyWith<_$PlotSpec_Plot3Impl> get copyWith =>
      __$$PlotSpec_Plot3ImplCopyWithImpl<_$PlotSpec_Plot3Impl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return plot3(series, axes);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return plot3?.call(series, axes);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (plot3 != null) {
      return plot3(series, axes);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return plot3(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return plot3?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (plot3 != null) {
      return plot3(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Plot3 extends PlotSpec {
  const factory PlotSpec_Plot3(
      {required final List<Series3D> series,
      required final Axes3D axes}) = _$PlotSpec_Plot3Impl;
  const PlotSpec_Plot3._() : super._();

  List<Series3D> get series;
  Axes3D get axes;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_Plot3ImplCopyWith<_$PlotSpec_Plot3Impl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PlotSpec_SubplotImplCopyWith<$Res> {
  factory _$$PlotSpec_SubplotImplCopyWith(_$PlotSpec_SubplotImpl value,
          $Res Function(_$PlotSpec_SubplotImpl) then) =
      __$$PlotSpec_SubplotImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int rows, int cols, List<SubplotCell> cells});
}

/// @nodoc
class __$$PlotSpec_SubplotImplCopyWithImpl<$Res>
    extends _$PlotSpecCopyWithImpl<$Res, _$PlotSpec_SubplotImpl>
    implements _$$PlotSpec_SubplotImplCopyWith<$Res> {
  __$$PlotSpec_SubplotImplCopyWithImpl(_$PlotSpec_SubplotImpl _value,
      $Res Function(_$PlotSpec_SubplotImpl) _then)
      : super(_value, _then);

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? rows = null,
    Object? cols = null,
    Object? cells = null,
  }) {
    return _then(_$PlotSpec_SubplotImpl(
      rows: null == rows
          ? _value.rows
          : rows // ignore: cast_nullable_to_non_nullable
              as int,
      cols: null == cols
          ? _value.cols
          : cols // ignore: cast_nullable_to_non_nullable
              as int,
      cells: null == cells
          ? _value._cells
          : cells // ignore: cast_nullable_to_non_nullable
              as List<SubplotCell>,
    ));
  }
}

/// @nodoc

class _$PlotSpec_SubplotImpl extends PlotSpec_Subplot {
  const _$PlotSpec_SubplotImpl(
      {required this.rows,
      required this.cols,
      required final List<SubplotCell> cells})
      : _cells = cells,
        super._();

  @override
  final int rows;
  @override
  final int cols;
  final List<SubplotCell> _cells;
  @override
  List<SubplotCell> get cells {
    if (_cells is EqualUnmodifiableListView) return _cells;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_cells);
  }

  @override
  String toString() {
    return 'PlotSpec.subplot(rows: $rows, cols: $cols, cells: $cells)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PlotSpec_SubplotImpl &&
            (identical(other.rows, rows) || other.rows == rows) &&
            (identical(other.cols, cols) || other.cols == cols) &&
            const DeepCollectionEquality().equals(other._cells, _cells));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, rows, cols, const DeepCollectionEquality().hash(_cells));

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PlotSpec_SubplotImplCopyWith<_$PlotSpec_SubplotImpl> get copyWith =>
      __$$PlotSpec_SubplotImplCopyWithImpl<_$PlotSpec_SubplotImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        line2D,
    required TResult Function(
            List<Series2D> series, Axes2D axes, Legend? legend)
        scatter2D,
    required TResult Function(
            List<String> categories, Float64List values, Axes2D axes)
        bar2D,
    required TResult Function(
            Float64List binEdges, Uint64List counts, Axes2D axes)
        histogram,
    required TResult Function(List<Series2D> series) polar,
    required TResult Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)
        quiver,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)
        surface,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)
        mesh,
    required TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)
        contour,
    required TResult Function(List<Series3D> series, Axes3D axes) plot3,
    required TResult Function(int rows, int cols, List<SubplotCell> cells)
        subplot,
  }) {
    return subplot(rows, cols, cells);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult? Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult? Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult? Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult? Function(List<Series2D> series)? polar,
    TResult? Function(Float64List x, Float64List y, Float64List u,
            Float64List v, Axes2D axes)?
        quiver,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult? Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult? Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult? Function(int rows, int cols, List<SubplotCell> cells)? subplot,
  }) {
    return subplot?.call(rows, cols, cells);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        line2D,
    TResult Function(List<Series2D> series, Axes2D axes, Legend? legend)?
        scatter2D,
    TResult Function(List<String> categories, Float64List values, Axes2D axes)?
        bar2D,
    TResult Function(Float64List binEdges, Uint64List counts, Axes2D axes)?
        histogram,
    TResult Function(List<Series2D> series)? polar,
    TResult Function(Float64List x, Float64List y, Float64List u, Float64List v,
            Axes2D axes)?
        quiver,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Shading shading, Colormap colormap)?
        surface,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Colormap colormap)?
        mesh,
    TResult Function(Float64List x, Float64List y, List<Float64List> z,
            Float64List levels)?
        contour,
    TResult Function(List<Series3D> series, Axes3D axes)? plot3,
    TResult Function(int rows, int cols, List<SubplotCell> cells)? subplot,
    required TResult orElse(),
  }) {
    if (subplot != null) {
      return subplot(rows, cols, cells);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(PlotSpec_Line2D value) line2D,
    required TResult Function(PlotSpec_Scatter2D value) scatter2D,
    required TResult Function(PlotSpec_Bar2D value) bar2D,
    required TResult Function(PlotSpec_Histogram value) histogram,
    required TResult Function(PlotSpec_Polar value) polar,
    required TResult Function(PlotSpec_Quiver value) quiver,
    required TResult Function(PlotSpec_Surface value) surface,
    required TResult Function(PlotSpec_Mesh value) mesh,
    required TResult Function(PlotSpec_Contour value) contour,
    required TResult Function(PlotSpec_Plot3 value) plot3,
    required TResult Function(PlotSpec_Subplot value) subplot,
  }) {
    return subplot(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(PlotSpec_Line2D value)? line2D,
    TResult? Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult? Function(PlotSpec_Bar2D value)? bar2D,
    TResult? Function(PlotSpec_Histogram value)? histogram,
    TResult? Function(PlotSpec_Polar value)? polar,
    TResult? Function(PlotSpec_Quiver value)? quiver,
    TResult? Function(PlotSpec_Surface value)? surface,
    TResult? Function(PlotSpec_Mesh value)? mesh,
    TResult? Function(PlotSpec_Contour value)? contour,
    TResult? Function(PlotSpec_Plot3 value)? plot3,
    TResult? Function(PlotSpec_Subplot value)? subplot,
  }) {
    return subplot?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(PlotSpec_Line2D value)? line2D,
    TResult Function(PlotSpec_Scatter2D value)? scatter2D,
    TResult Function(PlotSpec_Bar2D value)? bar2D,
    TResult Function(PlotSpec_Histogram value)? histogram,
    TResult Function(PlotSpec_Polar value)? polar,
    TResult Function(PlotSpec_Quiver value)? quiver,
    TResult Function(PlotSpec_Surface value)? surface,
    TResult Function(PlotSpec_Mesh value)? mesh,
    TResult Function(PlotSpec_Contour value)? contour,
    TResult Function(PlotSpec_Plot3 value)? plot3,
    TResult Function(PlotSpec_Subplot value)? subplot,
    required TResult orElse(),
  }) {
    if (subplot != null) {
      return subplot(this);
    }
    return orElse();
  }
}

abstract class PlotSpec_Subplot extends PlotSpec {
  const factory PlotSpec_Subplot(
      {required final int rows,
      required final int cols,
      required final List<SubplotCell> cells}) = _$PlotSpec_SubplotImpl;
  const PlotSpec_Subplot._() : super._();

  int get rows;
  int get cols;
  List<SubplotCell> get cells;

  /// Create a copy of PlotSpec
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PlotSpec_SubplotImplCopyWith<_$PlotSpec_SubplotImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$ScriptEvent {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(BigInt jobId, String stage, double? percent)
        progress,
    required TResult Function(BigInt jobId) cancelled,
    required TResult Function(EvalResult result) done,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(BigInt jobId, String stage, double? percent)? progress,
    TResult? Function(BigInt jobId)? cancelled,
    TResult? Function(EvalResult result)? done,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(BigInt jobId, String stage, double? percent)? progress,
    TResult Function(BigInt jobId)? cancelled,
    TResult Function(EvalResult result)? done,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ScriptEvent_Progress value) progress,
    required TResult Function(ScriptEvent_Cancelled value) cancelled,
    required TResult Function(ScriptEvent_Done value) done,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ScriptEvent_Progress value)? progress,
    TResult? Function(ScriptEvent_Cancelled value)? cancelled,
    TResult? Function(ScriptEvent_Done value)? done,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ScriptEvent_Progress value)? progress,
    TResult Function(ScriptEvent_Cancelled value)? cancelled,
    TResult Function(ScriptEvent_Done value)? done,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ScriptEventCopyWith<$Res> {
  factory $ScriptEventCopyWith(
          ScriptEvent value, $Res Function(ScriptEvent) then) =
      _$ScriptEventCopyWithImpl<$Res, ScriptEvent>;
}

/// @nodoc
class _$ScriptEventCopyWithImpl<$Res, $Val extends ScriptEvent>
    implements $ScriptEventCopyWith<$Res> {
  _$ScriptEventCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$ScriptEvent_ProgressImplCopyWith<$Res> {
  factory _$$ScriptEvent_ProgressImplCopyWith(_$ScriptEvent_ProgressImpl value,
          $Res Function(_$ScriptEvent_ProgressImpl) then) =
      __$$ScriptEvent_ProgressImplCopyWithImpl<$Res>;
  @useResult
  $Res call({BigInt jobId, String stage, double? percent});
}

/// @nodoc
class __$$ScriptEvent_ProgressImplCopyWithImpl<$Res>
    extends _$ScriptEventCopyWithImpl<$Res, _$ScriptEvent_ProgressImpl>
    implements _$$ScriptEvent_ProgressImplCopyWith<$Res> {
  __$$ScriptEvent_ProgressImplCopyWithImpl(_$ScriptEvent_ProgressImpl _value,
      $Res Function(_$ScriptEvent_ProgressImpl) _then)
      : super(_value, _then);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jobId = null,
    Object? stage = null,
    Object? percent = freezed,
  }) {
    return _then(_$ScriptEvent_ProgressImpl(
      jobId: null == jobId
          ? _value.jobId
          : jobId // ignore: cast_nullable_to_non_nullable
              as BigInt,
      stage: null == stage
          ? _value.stage
          : stage // ignore: cast_nullable_to_non_nullable
              as String,
      percent: freezed == percent
          ? _value.percent
          : percent // ignore: cast_nullable_to_non_nullable
              as double?,
    ));
  }
}

/// @nodoc

class _$ScriptEvent_ProgressImpl extends ScriptEvent_Progress {
  const _$ScriptEvent_ProgressImpl(
      {required this.jobId, required this.stage, this.percent})
      : super._();

  @override
  final BigInt jobId;
  @override
  final String stage;
  @override
  final double? percent;

  @override
  String toString() {
    return 'ScriptEvent.progress(jobId: $jobId, stage: $stage, percent: $percent)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ScriptEvent_ProgressImpl &&
            (identical(other.jobId, jobId) || other.jobId == jobId) &&
            (identical(other.stage, stage) || other.stage == stage) &&
            (identical(other.percent, percent) || other.percent == percent));
  }

  @override
  int get hashCode => Object.hash(runtimeType, jobId, stage, percent);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ScriptEvent_ProgressImplCopyWith<_$ScriptEvent_ProgressImpl>
      get copyWith =>
          __$$ScriptEvent_ProgressImplCopyWithImpl<_$ScriptEvent_ProgressImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(BigInt jobId, String stage, double? percent)
        progress,
    required TResult Function(BigInt jobId) cancelled,
    required TResult Function(EvalResult result) done,
  }) {
    return progress(jobId, stage, percent);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(BigInt jobId, String stage, double? percent)? progress,
    TResult? Function(BigInt jobId)? cancelled,
    TResult? Function(EvalResult result)? done,
  }) {
    return progress?.call(jobId, stage, percent);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(BigInt jobId, String stage, double? percent)? progress,
    TResult Function(BigInt jobId)? cancelled,
    TResult Function(EvalResult result)? done,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(jobId, stage, percent);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ScriptEvent_Progress value) progress,
    required TResult Function(ScriptEvent_Cancelled value) cancelled,
    required TResult Function(ScriptEvent_Done value) done,
  }) {
    return progress(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ScriptEvent_Progress value)? progress,
    TResult? Function(ScriptEvent_Cancelled value)? cancelled,
    TResult? Function(ScriptEvent_Done value)? done,
  }) {
    return progress?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ScriptEvent_Progress value)? progress,
    TResult Function(ScriptEvent_Cancelled value)? cancelled,
    TResult Function(ScriptEvent_Done value)? done,
    required TResult orElse(),
  }) {
    if (progress != null) {
      return progress(this);
    }
    return orElse();
  }
}

abstract class ScriptEvent_Progress extends ScriptEvent {
  const factory ScriptEvent_Progress(
      {required final BigInt jobId,
      required final String stage,
      final double? percent}) = _$ScriptEvent_ProgressImpl;
  const ScriptEvent_Progress._() : super._();

  BigInt get jobId;
  String get stage;
  double? get percent;

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ScriptEvent_ProgressImplCopyWith<_$ScriptEvent_ProgressImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ScriptEvent_CancelledImplCopyWith<$Res> {
  factory _$$ScriptEvent_CancelledImplCopyWith(
          _$ScriptEvent_CancelledImpl value,
          $Res Function(_$ScriptEvent_CancelledImpl) then) =
      __$$ScriptEvent_CancelledImplCopyWithImpl<$Res>;
  @useResult
  $Res call({BigInt jobId});
}

/// @nodoc
class __$$ScriptEvent_CancelledImplCopyWithImpl<$Res>
    extends _$ScriptEventCopyWithImpl<$Res, _$ScriptEvent_CancelledImpl>
    implements _$$ScriptEvent_CancelledImplCopyWith<$Res> {
  __$$ScriptEvent_CancelledImplCopyWithImpl(_$ScriptEvent_CancelledImpl _value,
      $Res Function(_$ScriptEvent_CancelledImpl) _then)
      : super(_value, _then);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? jobId = null,
  }) {
    return _then(_$ScriptEvent_CancelledImpl(
      jobId: null == jobId
          ? _value.jobId
          : jobId // ignore: cast_nullable_to_non_nullable
              as BigInt,
    ));
  }
}

/// @nodoc

class _$ScriptEvent_CancelledImpl extends ScriptEvent_Cancelled {
  const _$ScriptEvent_CancelledImpl({required this.jobId}) : super._();

  @override
  final BigInt jobId;

  @override
  String toString() {
    return 'ScriptEvent.cancelled(jobId: $jobId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ScriptEvent_CancelledImpl &&
            (identical(other.jobId, jobId) || other.jobId == jobId));
  }

  @override
  int get hashCode => Object.hash(runtimeType, jobId);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ScriptEvent_CancelledImplCopyWith<_$ScriptEvent_CancelledImpl>
      get copyWith => __$$ScriptEvent_CancelledImplCopyWithImpl<
          _$ScriptEvent_CancelledImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(BigInt jobId, String stage, double? percent)
        progress,
    required TResult Function(BigInt jobId) cancelled,
    required TResult Function(EvalResult result) done,
  }) {
    return cancelled(jobId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(BigInt jobId, String stage, double? percent)? progress,
    TResult? Function(BigInt jobId)? cancelled,
    TResult? Function(EvalResult result)? done,
  }) {
    return cancelled?.call(jobId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(BigInt jobId, String stage, double? percent)? progress,
    TResult Function(BigInt jobId)? cancelled,
    TResult Function(EvalResult result)? done,
    required TResult orElse(),
  }) {
    if (cancelled != null) {
      return cancelled(jobId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ScriptEvent_Progress value) progress,
    required TResult Function(ScriptEvent_Cancelled value) cancelled,
    required TResult Function(ScriptEvent_Done value) done,
  }) {
    return cancelled(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ScriptEvent_Progress value)? progress,
    TResult? Function(ScriptEvent_Cancelled value)? cancelled,
    TResult? Function(ScriptEvent_Done value)? done,
  }) {
    return cancelled?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ScriptEvent_Progress value)? progress,
    TResult Function(ScriptEvent_Cancelled value)? cancelled,
    TResult Function(ScriptEvent_Done value)? done,
    required TResult orElse(),
  }) {
    if (cancelled != null) {
      return cancelled(this);
    }
    return orElse();
  }
}

abstract class ScriptEvent_Cancelled extends ScriptEvent {
  const factory ScriptEvent_Cancelled({required final BigInt jobId}) =
      _$ScriptEvent_CancelledImpl;
  const ScriptEvent_Cancelled._() : super._();

  BigInt get jobId;

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ScriptEvent_CancelledImplCopyWith<_$ScriptEvent_CancelledImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ScriptEvent_DoneImplCopyWith<$Res> {
  factory _$$ScriptEvent_DoneImplCopyWith(_$ScriptEvent_DoneImpl value,
          $Res Function(_$ScriptEvent_DoneImpl) then) =
      __$$ScriptEvent_DoneImplCopyWithImpl<$Res>;
  @useResult
  $Res call({EvalResult result});
}

/// @nodoc
class __$$ScriptEvent_DoneImplCopyWithImpl<$Res>
    extends _$ScriptEventCopyWithImpl<$Res, _$ScriptEvent_DoneImpl>
    implements _$$ScriptEvent_DoneImplCopyWith<$Res> {
  __$$ScriptEvent_DoneImplCopyWithImpl(_$ScriptEvent_DoneImpl _value,
      $Res Function(_$ScriptEvent_DoneImpl) _then)
      : super(_value, _then);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? result = null,
  }) {
    return _then(_$ScriptEvent_DoneImpl(
      result: null == result
          ? _value.result
          : result // ignore: cast_nullable_to_non_nullable
              as EvalResult,
    ));
  }
}

/// @nodoc

class _$ScriptEvent_DoneImpl extends ScriptEvent_Done {
  const _$ScriptEvent_DoneImpl({required this.result}) : super._();

  @override
  final EvalResult result;

  @override
  String toString() {
    return 'ScriptEvent.done(result: $result)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ScriptEvent_DoneImpl &&
            (identical(other.result, result) || other.result == result));
  }

  @override
  int get hashCode => Object.hash(runtimeType, result);

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ScriptEvent_DoneImplCopyWith<_$ScriptEvent_DoneImpl> get copyWith =>
      __$$ScriptEvent_DoneImplCopyWithImpl<_$ScriptEvent_DoneImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(BigInt jobId, String stage, double? percent)
        progress,
    required TResult Function(BigInt jobId) cancelled,
    required TResult Function(EvalResult result) done,
  }) {
    return done(result);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(BigInt jobId, String stage, double? percent)? progress,
    TResult? Function(BigInt jobId)? cancelled,
    TResult? Function(EvalResult result)? done,
  }) {
    return done?.call(result);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(BigInt jobId, String stage, double? percent)? progress,
    TResult Function(BigInt jobId)? cancelled,
    TResult Function(EvalResult result)? done,
    required TResult orElse(),
  }) {
    if (done != null) {
      return done(result);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(ScriptEvent_Progress value) progress,
    required TResult Function(ScriptEvent_Cancelled value) cancelled,
    required TResult Function(ScriptEvent_Done value) done,
  }) {
    return done(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(ScriptEvent_Progress value)? progress,
    TResult? Function(ScriptEvent_Cancelled value)? cancelled,
    TResult? Function(ScriptEvent_Done value)? done,
  }) {
    return done?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(ScriptEvent_Progress value)? progress,
    TResult Function(ScriptEvent_Cancelled value)? cancelled,
    TResult Function(ScriptEvent_Done value)? done,
    required TResult orElse(),
  }) {
    if (done != null) {
      return done(this);
    }
    return orElse();
  }
}

abstract class ScriptEvent_Done extends ScriptEvent {
  const factory ScriptEvent_Done({required final EvalResult result}) =
      _$ScriptEvent_DoneImpl;
  const ScriptEvent_Done._() : super._();

  EvalResult get result;

  /// Create a copy of ScriptEvent
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ScriptEvent_DoneImplCopyWith<_$ScriptEvent_DoneImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
