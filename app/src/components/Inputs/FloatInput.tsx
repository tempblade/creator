import { useCallback, useState } from "react";
import { z } from "zod";

type FloatInputProps = {
  value: number;
  onChange: (value: number) => void;
  id: string;
};

const FloatInput: React.FC<FloatInputProps> = ({ value, onChange, id }) => {
  const [inputValue, setInputValue] = useState<string>(value.toString());

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = e.target.value.replace(",", ".");
      setInputValue(val);

      const nextValue = z.coerce.number().min(-9999).max(9999).safeParse(val);

      if (nextValue.success) {
        onChange(nextValue.data);
      }
    },
    [setInputValue, onChange]
  );

  return <input id={id} onChange={handleInputChange} value={inputValue} />;
};

export default FloatInput;
