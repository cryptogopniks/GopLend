import sqlite3, { Database } from 'sqlite3';

const db = new sqlite3.Database('./my_database.db', (err) => {
  if (err) {
    console.error('Ошибка при создании базы данных:', err);
  } else {
    // console.log('База данных успешно создана!');
  }
});

export async function setCollectionPriceResponseList(collectionPriceResponseList: any[]) {
  // Создание таблицы
  db.run(`
    CREATE TABLE IF NOT EXISTS collection_price_responses (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      data TEXT
    )
  `, (err) => {
    if (err) {
      console.error('Ошибка при создании таблицы:', err);
    } else {
    //   console.log('Таблица успешно создана!');
    }
  });

  // Преобразуем объект в JSON-строку
  const json = JSON.stringify(collectionPriceResponseList);

  // SQL-запрос для вставки данных
  const sql = 'INSERT INTO collection_price_responses (data) VALUES (?)';

  try {
    await new Promise<void>((resolve, reject) => {
      db.run(sql, [json], (err) => {
        if (err) {
          reject(err);
        } else {
          resolve();
        }
      });
    });
    // console.log('Данные успешно записаны в базу данных.');
  } catch (error) {
    // console.error('Ошибка при записи данных:', error);
  } finally {
    db.close();
  }
}


export async function readAllData() {
    const sql = 'SELECT * FROM collection_price_responses';
  
    try {
      const data: any[] = [];
  
      await new Promise<void>((resolve, reject) => {
        db.each(sql, (err: any, row: { data: string; }) => {
          if (err) {
            reject(err);
          } else {
            data.push(JSON.parse(row.data));
          }
        }, () => {
          resolve(); 
        });
      });
  
    return(data);
  
    } catch (error) {
      console.error('Ошибка при чтении данных:', error);
    } finally {
      db.close();
    }
  }

//   const collectionPriceResponseList = [{
//     "usdc": "1",
//     "stars": "2"
// }];

  //setCollectionPriceResponseList(collectionPriceResponseList);
  

  async function readAllDataFN() {
    console.log(await readAllData())
  }

  readAllDataFN()
