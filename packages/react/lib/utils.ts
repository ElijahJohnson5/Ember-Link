const emptyArray: unknown[] = [];

export const getEmptyArray = <T>() => {
  return emptyArray as Array<T>;
};
