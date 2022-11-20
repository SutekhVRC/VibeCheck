import "./Settings.css";

export default function () {
  return (
    <>
      <h2>Settings</h2>
      <div className="basic-form">
        <label>thing 1</label>
        <input></input>
        <label>thing 2</label>
        <input type={"checkbox"}></input>
        <label>thing 3</label>
        <select>
          <option>1</option>
          <option>2</option>
          <option>3</option>
        </select>
      </div>
    </>
  );
}
