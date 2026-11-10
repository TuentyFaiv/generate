import styles from "./my-example.module.css"; // @tfverse if styled

export interface MyExampleProps {
	title: string;
}

export function MyExample({ title }: MyExampleProps) {
	// @tfverse if styled
	return <h2 className={styles.myExample}>{title}</h2>;
	// @tfverse else
	return <h2 className="my-example">{title}</h2>;
	// @tfverse end
}
