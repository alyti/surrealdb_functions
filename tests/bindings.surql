DEFINE FUNCTION fn::number($number: number) {
    RETURN $number;
};

DEFINE FUNCTION fn::string($string: string) {
    RETURN $string;
};

DEFINE FUNCTION fn::bool($bool: bool) {
    RETURN $bool;
};

DEFINE FUNCTION fn::datetime($datetime: datetime) {
    RETURN $datetime;
};

DEFINE FUNCTION fn::duration($duration: duration) {
    RETURN $duration;
};

DEFINE FUNCTION fn::bytes($bytes: bytes) {
    RETURN $bytes;
};

DEFINE FUNCTION fn::uuid($uuid: uuid) {
    RETURN $uuid;
};

DEFINE FUNCTION fn::record($record: record) {
    RETURN $record;
};

DEFINE FUNCTION fn::int($int: int) {
    RETURN $int;
};

DEFINE FUNCTION fn::decimal($decimal: decimal) {
    RETURN $decimal;
};

DEFINE FUNCTION fn::point($point: point) {
    RETURN $point;
};

DEFINE FUNCTION fn::geometry($geometry: geometry) {
    RETURN $geometry;
};

DEFINE FUNCTION fn::array($array: array) {
    RETURN $array;
};

DEFINE FUNCTION fn::set($set: set) {
    RETURN $set;
};

DEFINE FUNCTION fn::object($object: object) {
    RETURN $object;
};

DEFINE FUNCTION fn::any($any: any) {
    RETURN $any;
};

DEFINE FUNCTION fn::either($any: bool | int) {
    RETURN $any;
};
